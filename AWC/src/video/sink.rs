//! Virtual camera sink.
//!
//! The sink owns a single writer thread and a one-frame slot.  Producers never
//! block: if a previous frame has not been written yet it is dropped and
//! counted.  That is the right policy for a live camera — a stale frame is
//! worse than a missed one, and blocking the decoder would only add latency.
//!
//! The Softcam DLL is loaded dynamically.  Registration (`regsvr32`) is a
//! separate, unchanged concern; loading the DLL does not register anything.

use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::frame::DecodedFrame;

#[derive(Clone, Debug, Default)]
pub struct SinkStats {
    pub sent: u64,
    pub dropped: u64,
    pub width: u32,
    pub height: u32,
    pub connected: bool,
    pub running: bool,
    pub last_seq: u64,
    pub last_pts_ms: u64,
    pub error: Option<String>,
}

pub trait VirtualCamSink: Send + Sync {
    fn start(&self) -> Result<(), String>;
    fn stop(&self);
    fn push_frame(&self, frame: DecodedFrame);
    fn is_running(&self) -> bool;
    fn stats(&self) -> SinkStats;
}

type CreateCameraFn = unsafe extern "system" fn(i32, i32, f32) -> *mut c_void;
type WaitForConnectionFn = unsafe extern "system" fn(*mut c_void, f32) -> bool;
type SendFrameFn = unsafe extern "system" fn(*mut c_void, *const u8);
type DeleteCameraFn = unsafe extern "system" fn(*mut c_void) -> bool;
type IsConnectedFn = unsafe extern "system" fn(*mut c_void) -> bool;

struct SoftcamApi {
    _lib: libloading::Library,
    create: CreateCameraFn,
    wait: WaitForConnectionFn,
    send: SendFrameFn,
    delete: DeleteCameraFn,
    is_connected: Option<IsConnectedFn>,
}

impl SoftcamApi {
    fn load() -> Result<Self, String> {
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Ok(path) = std::env::var("AWC_SOFTCAM_DLL") {
            candidates.push(PathBuf::from(path));
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                candidates.push(dir.join("softcam.dll"));
            }
        }
        candidates.push(PathBuf::from("softcam.dll"));
        // Development convenience: the Tauri client keeps the DLL next to its
        // source; reuse it instead of duplicating a binary blob in this crate.
        candidates.push(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../CLIENT/tauri-client/src-tauri/softcam.dll"),
        );

        let mut tried = Vec::new();
        for path in &candidates {
            if !path.exists() {
                tried.push(format!("{} (missing)", path.display()));
                continue;
            }
            let lib = match unsafe { libloading::Library::new(path) } {
                Ok(lib) => lib,
                Err(err) => {
                    tried.push(format!("{} ({err})", path.display()));
                    continue;
                }
            };
            match unsafe { Self::from_library(lib) } {
                Ok(api) => return Ok(api),
                Err(err) => tried.push(format!("{} ({err})", path.display())),
            }
        }
        Err(format!(
            "could not load softcam.dll; tried: {}",
            tried.join("; ")
        ))
    }

    unsafe fn from_library(lib: libloading::Library) -> Result<Self, String> {
        let create = unsafe { symbol(&lib, b"scCreateCamera\0")? };
        let wait = unsafe { symbol(&lib, b"scWaitForConnection\0")? };
        let send = unsafe { symbol(&lib, b"scSendFrame\0")? };
        let delete = unsafe { symbol(&lib, b"scDeleteCamera\0")? };
        let is_connected = unsafe { optional_symbol(&lib, b"scIsConnected\0") };
        Ok(Self {
            _lib: lib,
            create,
            wait,
            send,
            delete,
            is_connected,
        })
    }
}

unsafe fn symbol<T: Copy>(lib: &libloading::Library, name: &[u8]) -> Result<T, String> {
    let sym: libloading::Symbol<T> = unsafe { lib.get(name) }
        .map_err(|e| format!("missing export {}: {e}", String::from_utf8_lossy(name)))?;
    Ok(*sym)
}

unsafe fn optional_symbol<T: Copy>(lib: &libloading::Library, name: &[u8]) -> Option<T> {
    let sym: libloading::Symbol<T> = unsafe { lib.get(name) }.ok()?;
    Some(*sym)
}

struct Slot {
    frame: Option<DecodedFrame>,
}

struct SinkInner {
    running: AtomicBool,
    connected: AtomicBool,
    sent: AtomicU64,
    dropped: AtomicU64,
    width: AtomicU32,
    height: AtomicU32,
    last_seq: AtomicU64,
    last_pts_ms: AtomicU64,
    fps: AtomicU32,
    slot: Mutex<Slot>,
    wake: Condvar,
    error: Mutex<Option<String>>,
    thread: Mutex<Option<JoinHandle<()>>>,
    lifecycle: Mutex<()>,
}

impl SinkInner {
    fn fail(&self, message: impl Into<String>) {
        *self.error.lock().unwrap() = Some(message.into());
        self.running.store(false, Ordering::Release);
        self.connected.store(false, Ordering::Release);
    }
}

pub struct SoftcamSink {
    inner: Arc<SinkInner>,
}

impl SoftcamSink {
    pub fn new(fps: u32) -> Self {
        Self {
            inner: Arc::new(SinkInner {
                running: AtomicBool::new(false),
                connected: AtomicBool::new(false),
                sent: AtomicU64::new(0),
                dropped: AtomicU64::new(0),
                width: AtomicU32::new(0),
                height: AtomicU32::new(0),
                last_seq: AtomicU64::new(0),
                last_pts_ms: AtomicU64::new(0),
                fps: AtomicU32::new(fps.clamp(1, 120)),
                slot: Mutex::new(Slot { frame: None }),
                wake: Condvar::new(),
                error: Mutex::new(None),
                thread: Mutex::new(None),
                lifecycle: Mutex::new(()),
            }),
        }
    }
}

impl VirtualCamSink for SoftcamSink {
    fn start(&self) -> Result<(), String> {
        let _guard = self.inner.lifecycle.lock().unwrap();
        if self.inner.running.load(Ordering::Acquire) {
            return Ok(());
        }
        *self.inner.error.lock().unwrap() = None;
        *self.inner.slot.lock().unwrap() = Slot { frame: None };
        self.inner.running.store(true, Ordering::Release);

        let inner = Arc::clone(&self.inner);
        let handle = thread::Builder::new()
            .name("awc-softcam".into())
            .spawn(move || writer_loop(inner))
            .map_err(|e| format!("could not spawn softcam writer: {e}"))?;
        *self.inner.thread.lock().unwrap() = Some(handle);
        Ok(())
    }

    fn stop(&self) {
        let _guard = self.inner.lifecycle.lock().unwrap();
        if !self.inner.running.swap(false, Ordering::AcqRel) {
            return;
        }
        self.inner.wake.notify_all();
        if let Some(handle) = self.inner.thread.lock().unwrap().take() {
            // The writer waits in 100 ms slices and `scWaitForConnection` in
            // 200 ms slices, so this join is bounded to well under a second.
            let _ = handle.join();
        }
        self.inner.connected.store(false, Ordering::Release);
        self.inner.width.store(0, Ordering::Release);
        self.inner.height.store(0, Ordering::Release);
    }

    fn push_frame(&self, frame: DecodedFrame) {
        if !self.inner.running.load(Ordering::Acquire) {
            return;
        }
        let mut slot = self.inner.slot.lock().unwrap();
        if slot.frame.is_some() {
            self.inner.dropped.fetch_add(1, Ordering::Relaxed);
        }
        slot.frame = Some(frame);
        drop(slot);
        self.inner.wake.notify_one();
    }

    fn is_running(&self) -> bool {
        self.inner.running.load(Ordering::Acquire)
    }

    fn stats(&self) -> SinkStats {
        SinkStats {
            sent: self.inner.sent.load(Ordering::Relaxed),
            dropped: self.inner.dropped.load(Ordering::Relaxed),
            width: self.inner.width.load(Ordering::Relaxed),
            height: self.inner.height.load(Ordering::Relaxed),
            connected: self.inner.connected.load(Ordering::Relaxed),
            running: self.inner.running.load(Ordering::Acquire),
            last_seq: self.inner.last_seq.load(Ordering::Relaxed),
            last_pts_ms: self.inner.last_pts_ms.load(Ordering::Relaxed),
            error: self.inner.error.lock().unwrap().clone(),
        }
    }
}

fn writer_loop(inner: Arc<SinkInner>) {
    let api = match SoftcamApi::load() {
        Ok(api) => api,
        Err(err) => {
            inner.fail(err);
            return;
        }
    };

    let mut camera: Option<(*mut c_void, u32, u32)> = None;
    let fps = inner.fps.load(Ordering::Relaxed).max(1) as f32;

    while inner.running.load(Ordering::Acquire) {
        let frame = {
            let mut slot = inner.slot.lock().unwrap();
            while slot.frame.is_none() && inner.running.load(Ordering::Acquire) {
                let (guard, _) = inner
                    .wake
                    .wait_timeout(slot, Duration::from_millis(100))
                    .unwrap();
                slot = guard;
            }
            slot.frame.take()
        };
        let Some(frame) = frame else {
            continue;
        };

        let size_changed = camera
            .map(|(_, w, h)| w != frame.width || h != frame.height)
            .unwrap_or(true);
        if size_changed {
            if let Some((old, _, _)) = camera.take() {
                unsafe { (api.delete)(old) };
            }
            let cam = unsafe {
                (api.create)(
                    frame.width as i32,
                    frame.height as i32,
                    fps,
                )
            };
            if cam.is_null() {
                inner.fail(format!(
                    "scCreateCamera({}x{}) returned null",
                    frame.width, frame.height
                ));
                break;
            }
            inner.width.store(frame.width, Ordering::Relaxed);
            inner.height.store(frame.height, Ordering::Relaxed);
            inner.connected.store(false, Ordering::Release);

            // Wait briefly for a consumer (Zoom/Teams/OBS), in short slices so
            // Stop can interrupt us.  If nobody is watching yet we still start
            // sending: the DLL keeps the latest frame for the first client.
            let mut waited_ms = 0;
            while inner.running.load(Ordering::Acquire) && waited_ms < 1000 {
                if unsafe { (api.wait)(cam, 0.2) } {
                    inner.connected.store(true, Ordering::Release);
                    break;
                }
                waited_ms += 200;
            }
            camera = Some((cam, frame.width, frame.height));
        }

        let Some((cam, _, _)) = camera else {
            continue;
        };
        if !inner.running.load(Ordering::Acquire) {
            break;
        }

        unsafe { (api.send)(cam, frame.data.as_ptr()) };
        inner.sent.fetch_add(1, Ordering::Relaxed);
        inner.last_seq.store(frame.seq, Ordering::Relaxed);
        inner
            .last_pts_ms
            .store(frame.pts.as_millis() as u64, Ordering::Relaxed);
        if let Some(is_connected) = api.is_connected {
            let connected = unsafe { is_connected(cam) };
            inner.connected.store(connected, Ordering::Release);
        }
    }

    if let Some((cam, _, _)) = camera.take() {
        unsafe { (api.delete)(cam) };
    }
    inner.connected.store(false, Ordering::Release);
    inner.width.store(0, Ordering::Relaxed);
    inner.height.store(0, Ordering::Relaxed);
    inner.running.store(false, Ordering::Release);
}
