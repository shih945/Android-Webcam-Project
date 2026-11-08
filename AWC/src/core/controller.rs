//! Controller thread: owns the stream receiver and the virtual-camera sink.
//!
//! Every potentially blocking operation (HTTP feature query, ADB, register,
//! receiver stop) happens here, never on the egui thread.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::core::state::{LogLevel, SharedState};
use crate::core::{adb, installer, phone};
use crate::video::{
    PreviewConfig, SoftcamSink, SourceKind, StreamReceiver, StreamRequest,
    TestPatternRequest, TestPatternSource, VirtualCamSink,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Usb,
    Wifi,
}

pub enum Command {
    Connect {
        mode: Mode,
        ip: String,
        http_port: u16,
        rtsp_port: u16,
    },
    Disconnect,
    StartCam,
    StopCam,
    StartTestPattern {
        width: u32,
        height: u32,
        fps: u32,
    },
    StopTestPattern,
    SetResolution(String),
    SwitchCamera,
    SetRotation(u16),
    SetMirror(bool),
    RefreshDevices,
    ConnectDevice(String),
    RunSetup,
    UpdatePreview(PreviewConfig),
    Shutdown,
}

pub struct ControllerHandle {
    tx: Sender<Command>,
    join: Option<JoinHandle<()>>,
    pub state: Arc<SharedState>,
}

impl ControllerHandle {
    pub fn spawn_with_config(config: crate::core::config::Config) -> Self {
        let state = SharedState::new(config.preview());
        state.rotation.store(config.rotation, Ordering::Relaxed);
        state.mirror.store(config.mirror, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        let controller_state = Arc::clone(&state);
        let join = thread::Builder::new()
            .name("awc-controller".into())
            .spawn(move || controller_loop(rx, controller_state))
            .ok();
        Self { tx, join, state }
    }

    pub fn send(&self, command: Command) {
        let _ = self.tx.send(command);
    }

    pub fn shutdown(&mut self) {
        let _ = self.tx.send(Command::Shutdown);
        if let Some(join) = self.join.take() {
            // Give the controller a moment to stop the receiver; the process is
            // about to exit, so a detached thread is acceptable here.
            let (done_tx, done_rx) = mpsc::channel();
            thread::spawn(move || {
                let _ = join.join();
                let _ = done_tx.send(());
            });
            let _ = done_rx.recv_timeout(std::time::Duration::from_secs(3));
        }
    }
}

impl Drop for ControllerHandle {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Shutdown);
    }
}

fn stop_receiver(receiver: &mut Option<StreamReceiver>) {
    if let Some(mut current) = receiver.take() {
        current.stop();
    }
}

fn stop_test_pattern(source: &mut Option<TestPatternSource>, state: &SharedState) {
    if let Some(mut current) = source.take() {
        current.stop();
    }
    state.test_running.store(false, Ordering::Relaxed);
}

fn controller_loop(rx: Receiver<Command>, state: Arc<SharedState>) {
    let sink: Arc<SoftcamSink> = Arc::clone(&state.sink);
    let sink_trait: Arc<dyn VirtualCamSink> = sink.clone();
    let preview = Arc::clone(&state.preview);
    let stream = Arc::clone(&state.stream);
    let mut receiver: Option<StreamReceiver> = None;
    let mut test_source: Option<TestPatternSource> = None;
    let mut active_host: Option<(String, u16)> = None;

    while let Ok(command) = rx.recv() {
        match command {
            Command::Connect {
                mode,
                ip,
                http_port,
                rtsp_port,
            } => {
                stop_receiver(&mut receiver);
                stop_test_pattern(&mut test_source, &state);

                let host = match mode {
                    Mode::Usb => "127.0.0.1".to_string(),
                    Mode::Wifi => ip,
                };
                active_host = Some((host.clone(), http_port));
                state.set_connecting(format!("Querying {host}:{http_port}/features ..."));
                state.info(format!("Connecting to {host}:{http_port}"));

                match phone::fetch_features(&host, http_port) {
                    Ok(features) => {
                        let metadata = format!(
                            "{} · {} resolution(s)",
                            features.stream_protocol,
                            features.resolutions.len()
                        );
                        if let Some(first) = features.resolutions.first() {
                            *state.selected_resolution.lock().unwrap() = Some(first.clone());
                        }
                        *state.features.lock().unwrap() = Some(features.clone());
                        state.info(format!(
                            "Features: {} resolution(s), protocol {}",
                            features.resolutions.len(),
                            features.stream_protocol
                        ));

                        let rtsp_port = features.rtsp_port.unwrap_or(rtsp_port);
                        let (kind, url, protocol) = if phone::is_rtsp(&features) {
                            (
                                SourceKind::Rtsp,
                                format!("rtsp://{host}:{rtsp_port}"),
                                "RTSP",
                            )
                        } else {
                            (
                                SourceKind::Mjpeg,
                                format!("http://{host}:{http_port}/video"),
                                "MJPEG",
                            )
                        };

                        state.set_streaming(
                            format!("Live · {metadata}"),
                            protocol,
                            &url,
                        );
                        state.info(format!("Starting {protocol} stream: {url}"));
                        receiver = Some(StreamReceiver::start(
                            StreamRequest {
                                kind,
                                url,
                                hw_decode: true,
                            },
                            sink_trait.clone(),
                            Arc::clone(&preview),
                            Arc::clone(&stream),
                            Arc::clone(&state.rotation),
                            Arc::clone(&state.mirror),
                        ));
                    }
                    Err(err) => {
                        state.set_error(format!("Connection failed: {err}"));
                        state.error(format!("Connection failed: {err}"));
                    }
                }
            }

            Command::Disconnect => {
                stop_receiver(&mut receiver);
                stop_test_pattern(&mut test_source, &state);
                active_host = None;
                state.set_idle("Disconnected");
                state.info("Disconnected");
            }

            Command::StartCam => match sink.start() {
                Ok(()) => {
                    state.info("Virtual camera started");
                }
                Err(err) => state.error(format!("Virtual camera: {err}")),
            },

            Command::StopCam => {
                sink.stop();
                state.info("Virtual camera stopped");
            }

            Command::StartTestPattern {
                width,
                height,
                fps,
            } => {
                stop_receiver(&mut receiver);
                stop_test_pattern(&mut test_source, &state);
                *stream.last_error.lock().unwrap() = None;
                state.set_streaming(
                    format!("Test pattern · {width}×{height} @ {fps} fps"),
                    "TEST",
                    "test-pattern://local",
                );
                state.info(format!(
                    "Test pattern started: {width}×{height} @ {fps} fps"
                ));
                test_source = Some(TestPatternSource::start(
                    TestPatternRequest {
                        width,
                        height,
                        fps,
                    },
                    Arc::clone(&sink_trait),
                    Arc::clone(&preview),
                    Arc::clone(&stream),
                    Arc::clone(&state.rotation),
                    Arc::clone(&state.mirror),
                ));
                state.test_running.store(true, Ordering::Relaxed);
            }

            Command::StopTestPattern => {
                stop_test_pattern(&mut test_source, &state);
                state.set_idle("Test pattern stopped");
                state.info("Test pattern stopped");
            }

            Command::SetResolution(resolution) => {
                let Some((host, port)) = active_host.as_ref() else {
                    state.warn("Pick a resolution after connecting");
                    continue;
                };
                let mut query = format!(
                    "resolution_str={}",
                    resolution.replace(' ', "%20")
                );
                // The Android endpoint defaults an omitted camera to "back".
                // Preserve the current facing while changing resolution.
                if let Ok(current) = phone::fetch_settings(host, *port) {
                    query.push_str("&camera=");
                    query.push_str(&current.camera);
                }
                match phone::send_control(host, *port, &query) {
                    Ok(()) => {
                        *state.selected_resolution.lock().unwrap() = Some(resolution.clone());
                        state.info(format!("Resolution requested: {resolution}"));
                    }
                    Err(err) => state.error(format!("Resolution change failed: {err}")),
                }
            }

            Command::SwitchCamera => {
                let Some((host, port)) = active_host.as_ref() else {
                    state.warn("Connect to the phone first");
                    continue;
                };
                let current = match phone::fetch_settings(host, *port) {
                    Ok(settings) => settings,
                    Err(err) => {
                        state.error(format!("Camera switch failed: {err}"));
                        continue;
                    }
                };
                let target = if current.camera.eq_ignore_ascii_case("front") {
                    "back"
                } else {
                    "front"
                };
                let query = format!("camera={target}");
                match phone::send_control(host, *port, &query) {
                    Ok(()) => state.info(format!("Camera: {target}")),
                    Err(err) => state.error(format!("Camera switch failed: {err}")),
                }
            }

            Command::SetRotation(degrees) => {
                let degrees = match degrees {
                    90 => 90,
                    180 => 180,
                    270 => 270,
                    _ => 0,
                };
                state.rotation.store(degrees, Ordering::Relaxed);
                state.info(format!("Rotation: {degrees}°"));
            }

            Command::SetMirror(enabled) => {
                state.mirror.store(enabled, Ordering::Relaxed);
                state.info(format!(
                    "Mirror: {}",
                    if enabled { "on" } else { "off" }
                ));
            }

            Command::RefreshDevices => {
                state.info("Scanning ADB devices...");
                match adb::list_devices() {
                    Ok(devices) => {
                        let count = devices.len();
                        *state.devices.lock().unwrap() = devices;
                        state.info(format!("ADB: {count} device(s)"));
                    }
                    Err(err) => state.error(format!("ADB: {err}")),
                }
            }

            Command::ConnectDevice(device_id) => {
                match adb::forward_device(&device_id) {
                    Ok(()) => {
                        *state.selected_device.lock().unwrap() = Some(device_id.clone());
                        state.info(format!("ADB: forwarded 8080/8554 for {device_id}"));
                    }
                    Err(err) => state.error(format!("ADB forward failed: {err}")),
                }
            }

            Command::RunSetup => {
                installer::run_setup(&state);
                state
                    .registered
                    .store(installer::is_registered(), Ordering::Relaxed);
            }

            Command::UpdatePreview(config) => {
                state.preview.configure(&config);
                state.info(format!(
                    "Preview {} · {} fps · {}x{}",
                    if config.enabled { "on" } else { "off" },
                    config.fps,
                    config.max_width,
                    config.max_height
                ));
            }

            Command::Shutdown => break,
        }
    }

    stop_receiver(&mut receiver);
    stop_test_pattern(&mut test_source, &state);
    sink.stop();
    state.log(LogLevel::Info, "Controller stopped");
}
