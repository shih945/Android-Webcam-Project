//! Decoded video frames and their metadata.
//!
//! Every frame that reaches the virtual camera sink is a packed BGR24 buffer
//! (`len == width * height * 3`).  Sources are responsible for removing any
//! row padding before constructing a `DecodedFrame`; keeping one canonical
//! layout is what lets the Windows and Linux sinks stay dumb.

use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameSize {
    pub width: u32,
    pub height: u32,
}

impl FrameSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn packed_len(&self) -> usize {
        self.width as usize * self.height as usize * 3
    }
}

/// A decoded frame on its way to the virtual camera (and, optionally, the
/// preview tap).  The buffer is owned; the sink consumes the frame so the
/// writer never has to copy it again.
#[derive(Debug)]
pub struct DecodedFrame {
    /// Packed BGR24.
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// Presentation time relative to the start of the stream.  MJPEG uses the
    /// monotonic arrival time; RTSP uses the container PTS converted to a
    /// duration, falling back to arrival time when PTS is missing.
    pub pts: Duration,
    /// Monotonic frame sequence number, assigned by the source.
    pub seq: u64,
}

impl DecodedFrame {
    pub fn new_bgr(
        data: Vec<u8>,
        width: u32,
        height: u32,
        pts: Duration,
        seq: u64,
    ) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        if data.len() != FrameSize::new(width, height).packed_len() {
            return None;
        }
        Some(Self {
            data,
            width,
            height,
            pts,
            seq,
        })
    }

    /// Copy a source plane (which may have row padding) into a packed BGR
    /// buffer.  This is the single place that deals with `linesize`.
    pub fn pack_bgr_rows(
        src: &[u8],
        stride: usize,
        width: u32,
        height: u32,
    ) -> Option<Vec<u8>> {
        let row_bytes = width as usize * 3;
        if stride < row_bytes {
            return None;
        }
        let needed = stride.checked_mul(height as usize)?;
        if src.len() < needed {
            return None;
        }
        let mut out = vec![0u8; row_bytes * height as usize];
        for y in 0..height as usize {
            let start = y * stride;
            out[y * row_bytes..(y + 1) * row_bytes]
                .copy_from_slice(&src[start..start + row_bytes]);
        }
        Some(out)
    }
}
