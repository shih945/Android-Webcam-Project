//! The UI-independent video pipeline.
//!
//! ```text
//! StreamReceiver (network + decode)  ->  PreviewTap (downsample, optional)
//!                                    ->  SoftcamSink (write thread) -> softcam.dll
//! ```
//!
//! The UI only ever sees `PreviewFrame` (small RGBA) and counters.  Raw decoded
//! frames never leave this module.

pub mod frame;
pub mod preview;
pub mod sink;
pub mod source;
pub mod test_pattern;

pub use preview::{PreviewConfig, PreviewMode, PreviewTap};
pub use sink::{SinkStats, SoftcamSink, VirtualCamSink};
pub use source::{SourceKind, StreamReceiver, StreamRequest, StreamStats};
pub use test_pattern::{TestPatternRequest, TestPatternSource};
