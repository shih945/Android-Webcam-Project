//! Preview side-path.
//!
//! `PreviewTap::maybe_tap` runs on the decode thread, right before the frame is
//! handed to the virtual camera sink.  When preview is disabled it returns
//! before touching the frame, so the main path pays nothing.  When enabled it
//! downsamples to a small RGBA image and stores it in a single slot: the UI
//! takes the latest image and drops the old one.  There is deliberately no
//! queue, so a stalled UI can never slow down or grow memory on the video path.

use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::frame::DecodedFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PreviewMode {
    /// Downsample and throttle to `fps` (default).
    Throttled = 0,
    /// Downsample every decoded frame (no throttle).
    FullRate = 1,
    /// Full resolution, every decoded frame.
    Native = 2,
}

impl PreviewMode {
    fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::FullRate,
            2 => Self::Native,
            _ => Self::Throttled,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Throttled => "throttled",
            Self::FullRate => "fullrate",
            Self::Native => "native",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "fullrate" | "full-rate" => Self::FullRate,
            "native" | "full" => Self::Native,
            _ => Self::Throttled,
        }
    }
}

impl Default for PreviewMode {
    fn default() -> Self {
        Self::Throttled
    }
}

#[derive(Debug, Clone)]
pub struct PreviewConfig {
    pub enabled: bool,
    pub mode: PreviewMode,
    pub fps: u32,
    /// Upper bound for the preview; aspect ratio is preserved.
    pub max_width: u32,
    pub max_height: u32,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: PreviewMode::Throttled,
            fps: 8,
            max_width: 640,
            max_height: 360,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PreviewStats {
    pub taps: u64,
    pub last_micros: u64,
    pub total_micros: u64,
}

impl PreviewStats {
    pub fn average_micros(self) -> f64 {
        if self.taps == 0 {
            0.0
        } else {
            self.total_micros as f64 / self.taps as f64
        }
    }
}

#[derive(Debug)]
pub struct PreviewFrame {
    pub width: u32,
    pub height: u32,
    /// RGBA8, `width * height * 4` bytes.
    pub rgba: Vec<u8>,
}

pub struct PreviewTap {
    enabled: AtomicBool,
    mode: AtomicU8,
    target_fps: AtomicU32,
    max_width: AtomicU32,
    max_height: AtomicU32,
    last_emit: Mutex<Option<Instant>>,
    slot: Mutex<Option<PreviewFrame>>,
    taps: AtomicU64,
    last_tap_micros: AtomicU64,
    total_tap_micros: AtomicU64,
}

impl PreviewTap {
    pub fn new(config: PreviewConfig) -> Self {
        Self {
            enabled: AtomicBool::new(config.enabled),
            mode: AtomicU8::new(config.mode as u8),
            target_fps: AtomicU32::new(config.fps.clamp(1, 30)),
            max_width: AtomicU32::new(config.max_width.clamp(160, 1920)),
            max_height: AtomicU32::new(config.max_height.clamp(90, 1080)),
            last_emit: Mutex::new(None),
            slot: Mutex::new(None),
            taps: AtomicU64::new(0),
            last_tap_micros: AtomicU64::new(0),
            total_tap_micros: AtomicU64::new(0),
        }
    }

    pub fn configure(&self, config: &PreviewConfig) {
        let old_mode = self.mode.load(Ordering::Relaxed);
        self.enabled.store(config.enabled, Ordering::Relaxed);
        self.mode.store(config.mode as u8, Ordering::Relaxed);
        self.target_fps
            .store(config.fps.clamp(1, 30), Ordering::Relaxed);
        self.max_width
            .store(config.max_width.clamp(160, 1920), Ordering::Relaxed);
        self.max_height
            .store(config.max_height.clamp(90, 1080), Ordering::Relaxed);
        if !config.enabled {
            // Release the last image immediately when the user turns preview off.
            *self.slot.lock().unwrap() = None;
            *self.last_emit.lock().unwrap() = None;
        } else if old_mode != config.mode as u8 {
            // Avoid a stale throttle deadline when switching modes.
            *self.last_emit.lock().unwrap() = None;
        }
    }

    pub fn mode(&self) -> PreviewMode {
        PreviewMode::from_u8(self.mode.load(Ordering::Relaxed))
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn stats(&self) -> PreviewStats {
        PreviewStats {
            taps: self.taps.load(Ordering::Relaxed),
            last_micros: self.last_tap_micros.load(Ordering::Relaxed),
            total_micros: self.total_tap_micros.load(Ordering::Relaxed),
        }
    }

    /// Called from the decode thread for every decoded frame.  The early
    /// returns are the "zero overhead when off / throttled" path.
    pub fn maybe_tap(&self, frame: &DecodedFrame) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }
        let started = Instant::now();
        let mode = self.mode();

        if mode == PreviewMode::Throttled {
            let fps = self.target_fps.load(Ordering::Relaxed).max(1);
            let interval = Duration::from_secs_f64(1.0 / fps as f64);
            let mut last = self.last_emit.lock().unwrap();
            let now = Instant::now();
            if let Some(prev) = *last {
                if now.duration_since(prev) < interval {
                    return;
                }
            }
            *last = Some(now);
        }

        let (dw, dh) = if mode == PreviewMode::Native {
            (frame.width, frame.height)
        } else {
            fit_preview(
                frame.width,
                frame.height,
                self.max_width.load(Ordering::Relaxed),
                self.max_height.load(Ordering::Relaxed),
            )
        };
        if dw == 0 || dh == 0 {
            return;
        }

        let rgba = if dw == frame.width && dh == frame.height {
            bgr_to_rgba(frame)
        } else {
            downscale_bgr_to_rgba(frame, dw, dh)
        };
        let preview = PreviewFrame {
            width: dw,
            height: dh,
            rgba,
        };
        *self.slot.lock().unwrap() = Some(preview);

        let micros = started.elapsed().as_micros() as u64;
        self.last_tap_micros.store(micros, Ordering::Relaxed);
        self.total_tap_micros.fetch_add(micros, Ordering::Relaxed);
        self.taps.fetch_add(1, Ordering::Relaxed);
    }

    /// The UI calls this once per repaint; it takes ownership and leaves the
    /// slot empty, so the next frame can be written without blocking.
    pub fn take(&self) -> Option<PreviewFrame> {
        self.slot.lock().unwrap().take()
    }
}

fn fit_preview(src_w: u32, src_h: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    if src_w == 0 || src_h == 0 {
        return (0, 0);
    }
    let scale = (max_w as f32 / src_w as f32)
        .min(max_h as f32 / src_h as f32)
        .min(1.0);
    let w = ((src_w as f32 * scale).round() as u32).max(1);
    let h = ((src_h as f32 * scale).round() as u32).max(1);
    (w, h)
}

/// Full-resolution BGR24 → RGBA8 (no scaling).
fn bgr_to_rgba(frame: &DecodedFrame) -> Vec<u8> {
    let pixels = frame.width as usize * frame.height as usize;
    let mut out = vec![0u8; pixels * 4];
    for (src, dst) in frame.data.chunks_exact(3).zip(out.chunks_exact_mut(4)) {
        dst[0] = src[2];
        dst[1] = src[1];
        dst[2] = src[0];
        dst[3] = 255;
    }
    out
}

/// Area-average downscale + BGR24 → RGBA8 in one pass.
fn downscale_bgr_to_rgba(frame: &DecodedFrame, dst_w: u32, dst_h: u32) -> Vec<u8> {
    let src_w = frame.width as usize;
    let src_h = frame.height as usize;
    let dst_w = dst_w as usize;
    let dst_h = dst_h as usize;
    let mut out = vec![0u8; dst_w * dst_h * 4];

    for dy in 0..dst_h {
        let sy0 = dy * src_h / dst_h;
        let sy1 = (((dy + 1) * src_h / dst_h).max(sy0 + 1)).min(src_h);
        for dx in 0..dst_w {
            let sx0 = dx * src_w / dst_w;
            let sx1 = (((dx + 1) * src_w / dst_w).max(sx0 + 1)).min(src_w);
            let count = (sy1 - sy0) * (sx1 - sx0);
            if count == 0 {
                continue;
            }

            let mut b = 0u32;
            let mut g = 0u32;
            let mut r = 0u32;
            for sy in sy0..sy1 {
                let row = sy * src_w * 3;
                for sx in sx0..sx1 {
                    let p = row + sx * 3;
                    b += frame.data[p] as u32;
                    g += frame.data[p + 1] as u32;
                    r += frame.data[p + 2] as u32;
                }
            }
            let o = (dy * dst_w + dx) * 4;
            out[o] = (r / count as u32) as u8;
            out[o + 1] = (g / count as u32) as u8;
            out[o + 2] = (b / count as u32) as u8;
            out[o + 3] = 255;
        }
    }

    out
}

/// Test-only helper: build a solid BGR frame without touching the decoder.
#[cfg(test)]
pub fn solid_frame(width: u32, height: u32, b: u8, g: u8, r: u8) -> DecodedFrame {
    let mut data = vec![0u8; width as usize * height as usize * 3];
    for px in data.chunks_exact_mut(3) {
        px[0] = b;
        px[1] = g;
        px[2] = r;
    }
    DecodedFrame::new_bgr(data, width, height, Duration::ZERO, 0).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_tap_allocates_nothing() {
        let tap = PreviewTap::new(PreviewConfig {
            enabled: false,
            ..Default::default()
        });
        let frame = solid_frame(1920, 1080, 1, 2, 3);
        tap.maybe_tap(&frame);
        assert!(tap.take().is_none());
        assert_eq!(tap.stats().taps, 0);
    }

    #[test]
    fn tap_downscales_and_preserves_aspect() {
        let tap = PreviewTap::new(PreviewConfig {
            enabled: true,
            mode: PreviewMode::Throttled,
            fps: 30,
            max_width: 640,
            max_height: 360,
        });
        let frame = solid_frame(1920, 1080, 10, 20, 30);
        tap.maybe_tap(&frame);
        let p = tap.take().expect("preview frame");
        assert_eq!((p.width, p.height), (640, 360));
        assert_eq!(&p.rgba[0..4], &[30, 20, 10, 255]);
    }

    #[test]
    fn native_mode_keeps_source_size_and_ignores_throttle() {
        let tap = PreviewTap::new(PreviewConfig {
            enabled: true,
            mode: PreviewMode::Native,
            fps: 1,
            max_width: 640,
            max_height: 360,
        });
        let frame = solid_frame(800, 600, 1, 2, 3);
        tap.maybe_tap(&frame);
        let first = tap.take().expect("native preview");
        assert_eq!((first.width, first.height), (800, 600));
        // Full-rate/native modes must not throttle the second frame.
        tap.maybe_tap(&frame);
        assert!(tap.take().is_some());
    }
}
