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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Rotation {
    #[default]
    None,
    Cw90,
    Cw180,
    Cw270,
}

impl Rotation {
    pub fn from_degrees(degrees: u16) -> Self {
        match degrees % 360 {
            90 => Self::Cw90,
            180 => Self::Cw180,
            270 => Self::Cw270,
            _ => Self::None,
        }
    }

    pub fn is_quarter_turn(self) -> bool {
        matches!(self, Self::Cw90 | Self::Cw270)
    }

    /// Rotate a packed BGR24 frame clockwise.  Metadata is preserved.
    pub fn apply(self, frame: DecodedFrame) -> DecodedFrame {
        if self == Self::None {
            return frame;
        }
        let src_w = frame.width as usize;
        let src_h = frame.height as usize;
        let (dst_w, dst_h) = if self.is_quarter_turn() {
            (src_h, src_w)
        } else {
            (src_w, src_h)
        };
        let mut data = vec![0u8; frame.data.len()];

        for sy in 0..src_h {
            for sx in 0..src_w {
                let (dx, dy) = match self {
                    Self::None => (sx, sy),
                    Self::Cw90 => (src_h - 1 - sy, sx),
                    Self::Cw180 => (src_w - 1 - sx, src_h - 1 - sy),
                    Self::Cw270 => (sy, src_w - 1 - sx),
                };
                let src = (sy * src_w + sx) * 3;
                let dst = (dy * dst_w + dx) * 3;
                data[dst..dst + 3].copy_from_slice(&frame.data[src..src + 3]);
            }
        }

        DecodedFrame {
            data,
            width: dst_w as u32,
            height: dst_h as u32,
            pts: frame.pts,
            seq: frame.seq,
        }
    }
}

impl DecodedFrame {
    /// Mirror a packed BGR24 frame left-to-right.  Metadata is preserved.
    pub fn flip_horizontal(frame: DecodedFrame) -> DecodedFrame {
        let width = frame.width as usize;
        let height = frame.height as usize;
        let mut data = vec![0u8; frame.data.len()];
        for y in 0..height {
            for x in 0..width {
                let src = (y * width + x) * 3;
                let dst = (y * width + (width - 1 - x)) * 3;
                data[dst..dst + 3].copy_from_slice(&frame.data[src..src + 3]);
            }
        }
        DecodedFrame {
            data,
            width: frame.width,
            height: frame.height,
            pts: frame.pts,
            seq: frame.seq,
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> DecodedFrame {
        // 2x2, one byte per pixel is enough for the mapping test.
        DecodedFrame {
            data: vec![
                1, 1, 1, 2, 2, 2, //
                3, 3, 3, 4, 4, 4,
            ],
            width: 2,
            height: 2,
            pts: Duration::from_millis(10),
            seq: 7,
        }
    }

    fn first_pixels(frame: &DecodedFrame) -> Vec<u8> {
        frame.data.chunks_exact(3).map(|p| p[0]).collect()
    }

    #[test]
    fn rotation_90_swaps_and_maps() {
        let out = Rotation::Cw90.apply(sample());
        assert_eq!((out.width, out.height), (2, 2));
        assert_eq!(first_pixels(&out), vec![3, 1, 4, 2]);
        assert_eq!(out.seq, 7);
    }

    #[test]
    fn rotation_180_and_270_map() {
        assert_eq!(first_pixels(&Rotation::Cw180.apply(sample())), vec![4, 3, 2, 1]);
        assert_eq!(first_pixels(&Rotation::Cw270.apply(sample())), vec![2, 4, 1, 3]);
        assert_eq!(Rotation::from_degrees(450), Rotation::Cw90);
    }

    #[test]
    fn horizontal_flip_mirrors_each_row() {
        let out = DecodedFrame::flip_horizontal(sample());
        assert_eq!(first_pixels(&out), vec![2, 1, 4, 3]);
        assert_eq!((out.width, out.height), (2, 2));
        assert_eq!(out.seq, 7);
    }
}
