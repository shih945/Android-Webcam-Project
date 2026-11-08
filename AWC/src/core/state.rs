//! Shared state between the controller thread and the egui thread.
//!
//! The UI only reads snapshots; it never holds a lock while painting and never
//! sees a decoded frame.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::video::{
    PreviewConfig, PreviewTap, SinkStats, SoftcamSink, StreamStats, VirtualCamSink,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnState {
    Idle,
    Connecting,
    Streaming,
    Error,
}

#[derive(Clone, Debug)]
pub struct ConnInfo {
    pub state: ConnState,
    pub message: String,
    pub protocol: Option<String>,
    pub url: Option<String>,
}

impl Default for ConnInfo {
    fn default() -> Self {
        Self {
            state: ConnState::Idle,
            message: "Ready to connect.".to_string(),
            protocol: None,
            url: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Features {
    pub resolutions: Vec<String>,
    pub stream_protocol: String,
    pub rtsp_port: Option<u16>,
    pub manual_focus: bool,
    pub has_zoom: bool,
    pub zoom_min: f32,
    pub zoom_max: f32,
    pub exposure_lower: i32,
    pub exposure_upper: i32,
}

#[derive(Clone, Debug)]
pub struct Device {
    pub id: String,
    pub model: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug)]
pub struct LogLine {
    pub at: Instant,
    pub level: LogLevel,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct UiSnapshot {
    pub conn: ConnInfo,
    pub devices: Vec<Device>,
    pub selected_device: Option<String>,
    pub selected_resolution: Option<String>,
    pub sink: SinkStats,
    pub stream_frames: u64,
    pub stream_bytes: u64,
    pub stream_errors: u64,
    pub stream_reconnects: u64,
    pub stream_error: Option<String>,
    pub stream_pts_ms: u64,
    pub preview_taps: u64,
    pub registered: bool,
    pub test_running: bool,
}

pub struct SharedState {
    pub conn: Mutex<ConnInfo>,
    pub features: Mutex<Option<Features>>,
    pub devices: Mutex<Vec<Device>>,
    pub selected_device: Mutex<Option<String>>,
    pub selected_resolution: Mutex<Option<String>>,
    pub logs: Mutex<VecDeque<LogLine>>,
    pub stream: Arc<StreamStats>,
    pub preview: Arc<PreviewTap>,
    pub sink: Arc<SoftcamSink>,
    pub registered: AtomicBool,
    pub test_running: AtomicBool,
}

impl SharedState {
    pub fn new(preview: PreviewConfig) -> Arc<Self> {
        let sink = Arc::new(SoftcamSink::new(30));
        Arc::new(Self {
            conn: Mutex::new(ConnInfo::default()),
            features: Mutex::new(None),
            devices: Mutex::new(Vec::new()),
            selected_device: Mutex::new(None),
            selected_resolution: Mutex::new(None),
            logs: Mutex::new(VecDeque::new()),
            stream: Arc::new(StreamStats::default()),
            preview: Arc::new(PreviewTap::new(preview)),
            sink,
            registered: AtomicBool::new(false),
            test_running: AtomicBool::new(false),
        })
    }

    pub fn log(&self, level: LogLevel, text: impl Into<String>) {
        let mut logs = self.logs.lock().unwrap();
        if logs.len() >= 300 {
            logs.pop_front();
        }
        logs.push_back(LogLine {
            at: Instant::now(),
            level,
            text: text.into(),
        });
    }

    pub fn info(&self, text: impl Into<String>) {
        self.log(LogLevel::Info, text);
    }

    pub fn warn(&self, text: impl Into<String>) {
        self.log(LogLevel::Warn, text);
    }

    pub fn error(&self, text: impl Into<String>) {
        self.log(LogLevel::Error, text);
    }

    pub fn set_connecting(&self, message: impl Into<String>) {
        let mut conn = self.conn.lock().unwrap();
        conn.state = ConnState::Connecting;
        conn.message = message.into();
        conn.protocol = None;
        conn.url = None;
    }

    pub fn set_streaming(&self, message: impl Into<String>, protocol: &str, url: &str) {
        let mut conn = self.conn.lock().unwrap();
        conn.state = ConnState::Streaming;
        conn.message = message.into();
        conn.protocol = Some(protocol.to_string());
        conn.url = Some(url.to_string());
    }

    pub fn set_idle(&self, message: impl Into<String>) {
        let mut conn = self.conn.lock().unwrap();
        conn.state = ConnState::Idle;
        conn.message = message.into();
        conn.protocol = None;
        conn.url = None;
    }

    pub fn set_error(&self, message: impl Into<String>) {
        let mut conn = self.conn.lock().unwrap();
        conn.state = ConnState::Error;
        conn.message = message.into();
        conn.protocol = None;
        conn.url = None;
    }

    pub fn snapshot(&self) -> UiSnapshot {
        let preview_stats = self.preview.stats();
        UiSnapshot {
            conn: self.conn.lock().unwrap().clone(),
            devices: self.devices.lock().unwrap().clone(),
            selected_device: self.selected_device.lock().unwrap().clone(),
            selected_resolution: self.selected_resolution.lock().unwrap().clone(),
            sink: self.sink.stats(),
            stream_frames: self.stream.frames.load(Ordering::Relaxed),
            stream_bytes: self.stream.bytes.load(Ordering::Relaxed),
            stream_errors: self.stream.decode_errors.load(Ordering::Relaxed),
            stream_reconnects: self.stream.reconnects.load(Ordering::Relaxed),
            stream_error: self.stream.last_error.lock().unwrap().clone(),
            stream_pts_ms: self.stream.last_pts_micros.load(Ordering::Relaxed) / 1000,
            preview_taps: preview_stats.taps,
            registered: self.registered.load(Ordering::Relaxed),
            test_running: self.test_running.load(Ordering::Relaxed),
        }
    }
}
