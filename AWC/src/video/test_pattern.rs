//! Synthetic test source.
//!
//! It feeds the exact same `DecodedFrame → PreviewTap / VirtualCamSink` path as
//! a phone stream, so rendering, pacing, preview downscaling and softcam output
//! can be exercised without a device.  The pattern contains colour bars, a
//! moving box, a scan line and a binary frame counter.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::frame::DecodedFrame;
use super::preview::PreviewTap;
use super::sink::VirtualCamSink;
use super::source::StreamStats;

#[derive(Clone, Copy, Debug)]
pub struct TestPatternRequest {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

impl Default for TestPatternRequest {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            fps: 30,
        }
    }
}

pub struct TestPatternSource {
    running: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl TestPatternSource {
    pub fn start(
        request: TestPatternRequest,
        sink: Arc<dyn VirtualCamSink>,
        preview: Arc<PreviewTap>,
        stats: Arc<StreamStats>,
    ) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let flag = Arc::clone(&running);
        let width = request.width.max(16);
        let height = request.height.max(16);
        let fps = request.fps.clamp(1, 120);

        let thread = thread::Builder::new()
            .name("awc-test-pattern".into())
            .spawn(move || {
                let started = Instant::now();
                let interval = Duration::from_secs_f64(1.0 / fps as f64);
                let mut seq = 0u64;
                while flag.load(Ordering::Acquire) {
                    let due = started + interval.mul_f64(seq as f64);
                    sleep_until(due, &flag);
                    if !flag.load(Ordering::Acquire) {
                        break;
                    }

                    let pts = started.elapsed();
                    let data = render_pattern(width, height, seq, pts);
                    stats.bytes.fetch_add(data.len() as u64, Ordering::Relaxed);
                    if let Some(frame) = DecodedFrame::new_bgr(data, width, height, pts, seq) {
                        stats.frames.fetch_add(1, Ordering::Relaxed);
                        stats
                            .last_pts_micros
                            .store(pts.as_micros() as u64, Ordering::Relaxed);
                        preview.maybe_tap(&frame);
                        sink.push_frame(frame);
                    }
                    seq = seq.wrapping_add(1);
                }
            })
            .ok();

        Self { running, thread }
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for TestPatternSource {
    fn drop(&mut self) {
        self.stop();
    }
}

fn sleep_until(due: Instant, running: &AtomicBool) {
    loop {
        if !running.load(Ordering::Acquire) {
            return;
        }
        let now = Instant::now();
        if now >= due {
            return;
        }
        let left = due - now;
        thread::sleep(left.min(Duration::from_millis(4)));
    }
}

/// Render one BGR24 frame.  Exposed for unit tests.
pub fn render_pattern(width: u32, height: u32, seq: u64, pts: Duration) -> Vec<u8> {
    let w = width.max(16) as usize;
    let h = height.max(16) as usize;
    let mut data = vec![0u8; w * h * 3];

    // SMPTE-ish vertical bars (BGR).
    let bars = [
        (255u8, 255u8, 255u8), // white
        (0, 255, 255),         // yellow
        (255, 255, 0),         // cyan
        (0, 255, 0),           // green
        (255, 0, 255),         // magenta
        (0, 0, 255),           // red
        (255, 0, 0),           // blue
    ];
    let bar_w = (w / bars.len()).max(1);
    let mut row = vec![0u8; w * 3];
    for x in 0..w {
        let (b, g, r) = bars[(x / bar_w).min(bars.len() - 1)];
        row[x * 3] = b;
        row[x * 3 + 1] = g;
        row[x * 3 + 2] = r;
    }

    let bottom = h * 4 / 5;
    for y in 0..h {
        let dst = &mut data[y * w * 3..(y + 1) * w * 3];
        if y < bottom {
            dst.copy_from_slice(&row);
        } else {
            for x in 0..w {
                let value = (x * 255 / w.max(1)) as u8;
                dst[x * 3] = value;
                dst[x * 3 + 1] = value;
                dst[x * 3 + 2] = value;
            }
        }
    }

    // Moving box.
    let box_size = (w / 10).max(16);
    let travel = w.saturating_sub(box_size).max(1);
    let phase = (seq as usize * 7) % (travel * 2);
    let box_x = if phase < travel { phase } else { travel * 2 - phase };
    let box_y = h / 2 - box_size / 2;
    fill_rect(&mut data, w, h, box_x, box_y, box_size, box_size, (255, 255, 255));

    // Vertical scan line.
    let line_x = (seq as usize * 13) % w;
    fill_rect(&mut data, w, h, line_x, 0, 3.min(w), h, (0, 0, 0));

    // Top progress bar for the current two-second window.
    let progress = (pts.as_millis() % 2000) as usize * w / 2000;
    fill_rect(&mut data, w, h, 0, 0, progress, 4, (0, 255, 0));

    // 32-bit binary frame counter, 2×16 grid at the bottom-left.
    let cell = 8usize;
    let gap = 2usize;
    for bit in 0..32usize {
        let col = bit % 16;
        let row_index = bit / 16;
        let x = 8 + col * (cell + gap);
        let y = h.saturating_sub((cell + gap) * 2 + 8) + row_index * (cell + gap);
        let value = if (seq >> bit) & 1 == 1 { 255 } else { 30 };
        fill_rect(&mut data, w, h, x, y, cell, cell, (value, value, value));
    }

    data
}

fn fill_rect(
    data: &mut [u8],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    rect_w: usize,
    rect_h: usize,
    color: (u8, u8, u8),
) {
    if x >= width || y >= height {
        return;
    }
    let x_end = (x + rect_w).min(width);
    let y_end = (y + rect_h).min(height);
    let (b, g, r) = color;
    for row in y..y_end {
        let base = row * width * 3;
        for col in x..x_end {
            let p = base + col * 3;
            data[p] = b;
            data[p + 1] = g;
            data[p + 2] = r;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_has_expected_size_and_animates() {
        let width = 320;
        let height = 180;
        let a = render_pattern(width, height, 0, Duration::ZERO);
        let b = render_pattern(width, height, 7, Duration::from_millis(233));
        assert_eq!(a.len(), width as usize * height as usize * 3);
        assert_eq!(b.len(), a.len());
        assert_ne!(a, b);
    }
}
