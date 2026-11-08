//! Stream sources.
//!
//! Both sources produce the same thing: packed BGR24 `DecodedFrame`s with a
//! monotonic PTS.  The decode thread owns the network/decoder and pushes the
//! frame into the virtual-camera sink and the preview tap.  It never touches
//! egui, a channel to the UI, or a WebView.

use std::io::Read;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use ffmpeg_next as ff;
use ffmpeg_sys_next as ffi;
use zune_jpeg::JpegDecoder;

use super::frame::DecodedFrame;
use super::preview::PreviewTap;
use super::sink::VirtualCamSink;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceKind {
    Mjpeg,
    Rtsp,
}

#[derive(Clone, Debug)]
pub struct StreamRequest {
    pub kind: SourceKind,
    pub url: String,
    pub hw_decode: bool,
}

#[derive(Default)]
pub struct StreamStats {
    pub frames: AtomicU64,
    pub bytes: AtomicU64,
    pub decode_errors: AtomicU64,
    pub reconnects: AtomicU64,
    pub last_pts_micros: AtomicU64,
    pub last_error: Mutex<Option<String>>,
}

impl StreamStats {
    fn fail(&self, message: impl Into<String>) {
        *self.last_error.lock().unwrap() = Some(message.into());
    }
}

pub struct StreamReceiver {
    running: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl StreamReceiver {
    pub fn start(
        request: StreamRequest,
        sink: Arc<dyn VirtualCamSink>,
        preview: Arc<PreviewTap>,
        stats: Arc<StreamStats>,
    ) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let flag = Arc::clone(&running);
        *stats.last_error.lock().unwrap() = None;

        let thread = thread::Builder::new()
            .name("awc-stream".into())
            .spawn(move || {
                match request.kind {
                    SourceKind::Mjpeg => {
                        run_mjpeg(&request, sink, preview, stats, flag);
                    }
                    SourceKind::Rtsp => {
                        run_rtsp(&request, sink, preview, stats, flag);
                    }
                }
            })
            .ok();

        Self { running, thread }
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Release);
        self.thread.take();
    }
}

impl Drop for StreamReceiver {
    fn drop(&mut self) {
        self.stop();
    }
}

// ---------------------------------------------------------------------------
// MJPEG
// ---------------------------------------------------------------------------

fn run_mjpeg(
    request: &StreamRequest,
    sink: Arc<dyn VirtualCamSink>,
    preview: Arc<PreviewTap>,
    stats: Arc<StreamStats>,
    running: Arc<AtomicBool>,
) {
    let client = match reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            stats.fail(format!("HTTP client: {err}"));
            return;
        }
    };

    let started = Instant::now();
    let mut seq = 0u64;
    let mut buffer: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 16384];

    while running.load(Ordering::Acquire) {
        let mut response = match client.get(&request.url).send() {
            Ok(response) if response.status().is_success() => response,
            Ok(response) => {
                stats.fail(format!("MJPEG HTTP {}", response.status()));
                sleep_unless_stopped(&running, Duration::from_secs(1));
                continue;
            }
            Err(err) => {
                stats.fail(format!("MJPEG connect: {err}"));
                sleep_unless_stopped(&running, Duration::from_secs(1));
                continue;
            }
        };

        loop {
            if !running.load(Ordering::Acquire) {
                return;
            }
            let Some(jpeg) = read_next_jpeg(&mut response, &mut buffer, &mut chunk) else {
                break;
            };
            seq = seq.wrapping_add(1);
            stats.bytes.fetch_add(jpeg.len() as u64, Ordering::Relaxed);

            // Still drain the socket when nothing consumes frames, so the
            // phone's send buffer does not back up.
            if !sink.is_running() && !preview.is_enabled() {
                continue;
            }

            match decode_jpeg_bgr(&jpeg) {
                Some((data, width, height)) => {
                    let pts = started.elapsed();
                    if let Some(frame) = DecodedFrame::new_bgr(data, width, height, pts, seq) {
                        stats.frames.fetch_add(1, Ordering::Relaxed);
                        stats
                            .last_pts_micros
                            .store(pts.as_micros() as u64, Ordering::Relaxed);
                        preview.maybe_tap(&frame);
                        sink.push_frame(frame);
                    } else {
                        stats.decode_errors.fetch_add(1, Ordering::Relaxed);
                    }
                }
                None => {
                    stats.decode_errors.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        if running.load(Ordering::Acquire) {
            stats.reconnects.fetch_add(1, Ordering::Relaxed);
            stats.fail("MJPEG stream ended; reconnecting");
            sleep_unless_stopped(&running, Duration::from_millis(800));
        }
    }
}

fn sleep_unless_stopped(running: &AtomicBool, duration: Duration) {
    let step = Duration::from_millis(50);
    let mut left = duration;
    while left > Duration::ZERO && running.load(Ordering::Acquire) {
        let s = if left > step { step } else { left };
        thread::sleep(s);
        left = left.saturating_sub(s);
    }
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn parse_content_length(header: &[u8]) -> Option<usize> {
    let text = String::from_utf8_lossy(header);
    for line in text.lines() {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case("content-length") {
            let value = value.trim();
            let len: usize = value.parse().ok()?;
            // Guard against a corrupt header asking for an unbounded allocation.
            if len == 0 || len > 32 * 1024 * 1024 {
                return None;
            }
            return Some(len);
        }
    }
    None
}

fn read_next_jpeg(
    reader: &mut impl Read,
    buffer: &mut Vec<u8>,
    chunk: &mut [u8],
) -> Option<Vec<u8>> {
    const HEADER_END: &[u8] = b"\r\n\r\n";
    let header_end = loop {
        if let Some(pos) = find_bytes(buffer, HEADER_END) {
            break pos;
        }
        if buffer.len() > 64 * 1024 {
            // A malformed header should not make us grow without bound.
            buffer.clear();
        }
        let n = reader.read(chunk).ok()?;
        if n == 0 {
            return None;
        }
        buffer.extend_from_slice(&chunk[..n]);
    };

    let content_length = parse_content_length(&buffer[..header_end])?;
    let frame_start = header_end + HEADER_END.len();
    let frame_end = frame_start.checked_add(content_length)?;
    while buffer.len() < frame_end {
        let n = reader.read(chunk).ok()?;
        if n == 0 {
            return None;
        }
        buffer.extend_from_slice(&chunk[..n]);
    }

    let jpeg = buffer[frame_start..frame_end].to_vec();
    buffer.drain(..frame_end);
    Some(jpeg)
}

fn decode_jpeg_bgr(jpeg: &[u8]) -> Option<(Vec<u8>, u32, u32)> {
    let mut decoder = JpegDecoder::new(jpeg);
    let pixels = decoder.decode().ok()?;
    let (width, height) = decoder.dimensions()?;
    if width == 0 || height == 0 {
        return None;
    }
    let pixels_len = width.checked_mul(height)?;
    if pixels.len() == pixels_len * 3 {
        let mut data = pixels;
        for px in data.chunks_exact_mut(3) {
            px.swap(0, 2); // RGB -> BGR
        }
        Some((data, width as u32, height as u32))
    } else if pixels.len() == pixels_len {
        // Grayscale cameras exist; expand rather than dropping every frame.
        let mut data = Vec::with_capacity(pixels_len * 3);
        for value in pixels {
            data.extend_from_slice(&[value, value, value]);
        }
        Some((data, width as u32, height as u32))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// RTSP / H.264
// ---------------------------------------------------------------------------

static HW_PIX_FMT: AtomicI32 = AtomicI32::new(i32::MIN);

unsafe extern "C" fn get_hw_format(
    _ctx: *mut ffi::AVCodecContext,
    pix_fmts: *const ffi::AVPixelFormat,
) -> ffi::AVPixelFormat {
    let want = HW_PIX_FMT.load(Ordering::Relaxed);
    let mut p = pix_fmts;
    unsafe {
        while *p != ffi::AVPixelFormat::AV_PIX_FMT_NONE {
            if (*p) as i32 == want {
                return *p;
            }
            p = p.add(1);
        }
    }
    ffi::AVPixelFormat::AV_PIX_FMT_NONE
}

unsafe fn try_attach_hw(
    decoder: &mut ff::decoder::Video,
    decoder_codec: &ff::Codec,
    hwaccel_type: ffi::AVHWDeviceType,
) -> bool {
    let mut index = 0;
    let mut matched_fmt: Option<ffi::AVPixelFormat> = None;
    loop {
        let config = unsafe { ffi::avcodec_get_hw_config(decoder_codec.as_ptr(), index) };
        if config.is_null() {
            break;
        }
        let cfg = unsafe { *config };
        if (cfg.methods & ffi::AV_CODEC_HW_CONFIG_METHOD_HW_DEVICE_CTX as i32) != 0
            && cfg.device_type == hwaccel_type
        {
            matched_fmt = Some(cfg.pix_fmt);
            break;
        }
        index += 1;
    }

    let Some(fmt) = matched_fmt else {
        return false;
    };

    let mut hw_device_ctx: *mut ffi::AVBufferRef = ptr::null_mut();
    let ret = unsafe {
        ffi::av_hwdevice_ctx_create(
            &mut hw_device_ctx,
            hwaccel_type,
            ptr::null(),
            ptr::null_mut(),
            0,
        )
    };
    if ret < 0 || hw_device_ctx.is_null() {
        return false;
    }

    HW_PIX_FMT.store(fmt as i32, Ordering::Relaxed);
    let ctx_ptr = decoder.as_mut_ptr();
    unsafe {
        (*ctx_ptr).hw_device_ctx = ffi::av_buffer_ref(hw_device_ctx);
        (*ctx_ptr).get_format = Some(get_hw_format);
        ffi::av_buffer_unref(&mut hw_device_ctx);
    }
    true
}

fn open_rtsp_stream(
    url: &str,
    use_hw: bool,
    hwaccel_type: ffi::AVHWDeviceType,
) -> Result<
    (
        ff::format::context::Input,
        usize,
        ff::decoder::Video,
        ff::Rational,
    ),
    ff::Error,
> {
    let mut opts = ff::Dictionary::new();
    opts.set("rtsp_transport", "tcp");
    // Socket timeout in microseconds; keeps a dead phone from hanging forever.
    opts.set("stimeout", "3000000");

    let ictx = ff::format::input_with_dictionary(url, opts)?;
    let video_stream = ictx
        .streams()
        .best(ff::media::Type::Video)
        .ok_or(ff::Error::StreamNotFound)?;
    let video_stream_index = video_stream.index();
    let time_base = video_stream.time_base();
    let codec_params = video_stream.parameters();

    let decoder_codec =
        ff::decoder::find(codec_params.id()).ok_or(ff::Error::DecoderNotFound)?;
    let context = ff::codec::context::Context::from_parameters(codec_params)?;
    let mut decoder = context.decoder().video()?;

    if use_hw {
        let attached = unsafe { try_attach_hw(&mut decoder, &decoder_codec, hwaccel_type) };
        if !attached {
            eprintln!("awc: hardware decoder attach failed; using software this run");
        }
    }

    Ok((ictx, video_stream_index, decoder, time_base))
}

fn pts_to_duration(
    pts: Option<i64>,
    time_base: ff::Rational,
    first: &mut Option<Duration>,
    fallback: Duration,
) -> Duration {
    let Some(value) = pts.filter(|v| *v >= 0) else {
        return fallback;
    };
    let num = time_base.numerator() as i128;
    let den = time_base.denominator() as i128;
    if num <= 0 || den <= 0 {
        return fallback;
    }
    let micros = value as i128 * num * 1_000_000 / den;
    if micros < 0 {
        return fallback;
    }
    let duration = Duration::from_micros(micros as u64);
    let base = *first.get_or_insert(duration);
    duration.saturating_sub(base)
}

fn run_rtsp(
    request: &StreamRequest,
    sink: Arc<dyn VirtualCamSink>,
    preview: Arc<PreviewTap>,
    stats: Arc<StreamStats>,
    running: Arc<AtomicBool>,
) {
    if let Err(err) = ff::init() {
        stats.fail(format!("ffmpeg init: {err}"));
        return;
    }

    #[cfg(target_os = "windows")]
    let hwaccel_type = ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_D3D11VA;
    #[cfg(not(target_os = "windows"))]
    let hwaccel_type = ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_NONE;

    let mut use_hw = request.hw_decode;
    let started = Instant::now();
    let mut seq = 0u64;

    while running.load(Ordering::Acquire) {
        let (mut ictx, video_stream_index, mut decoder, time_base) =
            match open_rtsp_stream(&request.url, use_hw, hwaccel_type) {
                Ok(value) => value,
                Err(err) => {
                    stats.fail(format!("RTSP open: {err}"));
                    stats.reconnects.fetch_add(1, Ordering::Relaxed);
                    sleep_unless_stopped(&running, Duration::from_secs(1));
                    continue;
                }
            };

        let mut first_pts: Option<Duration> = None;
        let mut sws: Option<(
            ff::format::Pixel,
            u32,
            u32,
            ff::software::scaling::Context,
        )> = None;
        let mut restart_stream = false;

        for (stream, packet) in ictx.packets() {
            if !running.load(Ordering::Acquire) {
                return;
            }
            if stream.index() != video_stream_index {
                continue;
            }

            if let Err(err) = decoder.send_packet(&packet) {
                if use_hw {
                    eprintln!("awc: hardware decode rejected the stream ({err}); retrying in software");
                    use_hw = false;
                    restart_stream = true;
                    break;
                }
                stats.fail(format!("RTSP decode: {err}"));
                stats.reconnects.fetch_add(1, Ordering::Relaxed);
                restart_stream = true;
                break;
            }

            let mut decoded = ff::util::frame::Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let frame_pts = decoded.pts();
                let mut sw_frame = ff::util::frame::Video::empty();
                let source_frame: &ff::util::frame::Video = if use_hw {
                    let ok = unsafe {
                        ffi::av_hwframe_transfer_data(
                            sw_frame.as_mut_ptr(),
                            decoded.as_ptr(),
                            0,
                        ) >= 0
                    };
                    if !ok {
                        stats.decode_errors.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                    &sw_frame
                } else {
                    &decoded
                };

                let width = source_frame.width();
                let height = source_frame.height();
                if width == 0 || height == 0 {
                    continue;
                }

                let source_format = source_frame.format();
                let rebuild = match &sws {
                    Some((format, w, h, _)) => {
                        *format != source_format || *w != width || *h != height
                    }
                    None => true,
                };
                if rebuild {
                    match ff::software::scaling::Context::get(
                        source_format,
                        width,
                        height,
                        ff::format::Pixel::BGR24,
                        width,
                        height,
                        ff::software::scaling::Flags::BILINEAR,
                    ) {
                        Ok(context) => sws = Some((source_format, width, height, context)),
                        Err(err) => {
                            stats.fail(format!("sws context: {err}"));
                            stats.decode_errors.fetch_add(1, Ordering::Relaxed);
                            restart_stream = true;
                            break;
                        }
                    }
                }

                let mut bgr_frame = ff::util::frame::Video::empty();
                let Some((_, _, _, scaler)) = sws.as_mut() else {
                    continue;
                };
                if let Err(err) = scaler.run(source_frame, &mut bgr_frame) {
                    stats.fail(format!("sws scale: {err}"));
                    stats.decode_errors.fetch_add(1, Ordering::Relaxed);
                    continue;
                }

                let Some(data) = DecodedFrame::pack_bgr_rows(
                    bgr_frame.data(0),
                    bgr_frame.stride(0),
                    width,
                    height,
                ) else {
                    stats.decode_errors.fetch_add(1, Ordering::Relaxed);
                    continue;
                };

                seq = seq.wrapping_add(1);
                let pts = pts_to_duration(frame_pts, time_base, &mut first_pts, started.elapsed());
                if let Some(frame) = DecodedFrame::new_bgr(data, width, height, pts, seq) {
                    stats.frames.fetch_add(1, Ordering::Relaxed);
                    stats
                        .last_pts_micros
                        .store(pts.as_micros() as u64, Ordering::Relaxed);
                    preview.maybe_tap(&frame);
                    sink.push_frame(frame);
                }
            }
            if restart_stream {
                break;
            }
        }

        if !running.load(Ordering::Acquire) {
            return;
        }
        if restart_stream || !use_hw {
            stats.reconnects.fetch_add(1, Ordering::Relaxed);
        }
        sleep_unless_stopped(&running, Duration::from_millis(800));
    }
}
