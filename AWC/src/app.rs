//! egui front-end.
//!
//! This layer only ever receives small RGBA preview frames (`PreviewFrame`) and
//! counters.  Decoded BGR frames stay inside `video::source` / `video::sink`.

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, RichText, TextureOptions};

use crate::core::config::Config;
use crate::core::controller::{Command, ControllerHandle, Mode};
use crate::core::installer;
use crate::core::state::{ConnState, LogLevel, SharedState, UiSnapshot};
use crate::video::PreviewMode;
use shadcn_egui::components::badge::{Badge, BadgeVariant};
use shadcn_egui::components::button::{Button, ButtonSize, ButtonVariant};
use shadcn_egui::components::card::{card_content, card_header, Card, CardSize};
use shadcn_egui::components::input::Input;
use shadcn_egui::components::switch::Switch;

const PREVIEW_PRESETS: &[(&str, u32, u32)] = &[
    ("360p (640×360)", 640, 360),
    ("480p (854×480)", 854, 480),
    ("720p (1280×720)", 1280, 720),
    ("360p 4:3 (480×360)", 480, 360),
];

const TEST_PRESETS: &[(&str, u32, u32)] = &[
    ("480p (854×480)", 854, 480),
    ("720p (1280×720)", 1280, 720),
    ("1080p (1920×1080)", 1920, 1080),
];

#[derive(Clone, Copy)]
struct Palette {
    canvas: Color32,
    sidebar: Color32,
    card: Color32,
    border: Color32,
    text: Color32,
    muted: Color32,
    green: Color32,
    red: Color32,
    amber: Color32,
}

impl Palette {
    fn from_visuals(v: &egui::Visuals) -> Self {
        let light = !v.dark_mode;
        let (canvas, sidebar, card, border, green, red, amber) = if light {
            (
                Color32::from_rgb(244, 244, 245),
                Color32::from_rgb(250, 250, 250),
                Color32::from_rgb(255, 255, 255),
                Color32::from_rgb(212, 212, 216),
                Color32::from_rgb(4, 120, 87),
                Color32::from_rgb(185, 28, 28),
                Color32::from_rgb(180, 83, 9),
            )
        } else {
            (
                Color32::from_rgb(9, 9, 11),
                Color32::from_rgb(15, 15, 17),
                Color32::from_rgb(24, 24, 27),
                Color32::from_rgb(39, 39, 42),
                Color32::from_rgb(52, 211, 153),
                Color32::from_rgb(248, 113, 113),
                Color32::from_rgb(251, 191, 36),
            )
        };
        Self {
            canvas,
            sidebar,
            card,
            border,
            text: v.text_color(),
            muted: v.weak_text_color(),
            green,
            red,
            amber,
        }
    }
}

pub struct AwcApp {
    controller: ControllerHandle,
    state: std::sync::Arc<SharedState>,
    config: Config,
    mode: Mode,
    ip: String,
    http_port: String,
    rtsp_port: String,
    preview_texture: Option<egui::TextureHandle>,
    show_log: bool,
    last_sample: Instant,
    last_frames: u64,
    last_sent: u64,
    last_dropped: u64,
    stream_fps: f32,
    sink_fps: f32,
    dropped_delta: u64,
    test_width: u32,
    test_height: u32,
    test_fps: u32,
    palette: Palette,
    theme_pref: egui::ThemePreference,
    preview_mode: PreviewMode,
    sidebar_tab: usize,
}

impl AwcApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        autostart_test: bool,
        autostart_cam: bool,
        no_preview: bool,
        wifi: Option<(String, u16)>,
    ) -> Self {
        let mut config = crate::core::config::load();
        if no_preview {
            config.preview_enabled = false;
        }
        if let Some((host, port)) = &wifi {
            config.mode = "wifi".to_string();
            config.ip = host.clone();
            config.http_port = *port;
            let _ = config.save();
        }
        if let Ok(mode) = std::env::var("AWC_PREVIEW_MODE") {
            config.preview_mode = mode;
        }
        if let Ok(fps) = std::env::var("AWC_PREVIEW_FPS") {
            if let Ok(fps) = fps.parse::<u32>() {
                config.preview_fps = fps;
            }
        }
        let mode = if config.mode.eq_ignore_ascii_case("wifi") {
            Mode::Wifi
        } else {
            Mode::Usb
        };
        let controller = ControllerHandle::spawn_with_config(config.clone());
        let state = std::sync::Arc::clone(&controller.state);
        let registered = installer::is_registered();
        state.registered.store(registered, Ordering::Relaxed);

        let theme_pref = match config.theme.as_str() {
            "light" => egui::ThemePreference::Light,
            "dark" => egui::ThemePreference::Dark,
            _ => egui::ThemePreference::System,
        };
        shadcn_egui::Theme::vega().apply(&cc.egui_ctx);
        cc.egui_ctx.set_theme(theme_pref);
        cc.egui_ctx.global_style_mut(|style| {
            let mut scroll = egui::style::ScrollStyle::solid();
            scroll.bar_width = 8.0;
            scroll.bar_inner_margin = 4.0;
            scroll.bar_outer_margin = 4.0;
            scroll.handle_min_length = 24.0;
            scroll.foreground_color = true;
            style.spacing.scroll = scroll;
        });

        if !registered {
            state.warn("Virtual camera not registered — press Setup to install it");
        } else {
            state.info("AWC ready");
        }
        if mode == Mode::Usb {
            controller.send(Command::RefreshDevices);
        }

        // The initial viewport is deliberately small enough for a 1366×768
        // display at 150% scaling; users can maximise or resize it.
        let (test_width, test_height, test_fps) =
            (config.test_width, config.test_height, config.test_fps);
        let preview_mode = PreviewMode::parse(&config.preview_mode);

        if autostart_test {
            controller.send(Command::StartTestPattern {
                width: test_width,
                height: test_height,
                fps: test_fps,
            });
        }
        if autostart_cam {
            controller.send(Command::StartCam);
        }
        if let Some((host, port)) = &wifi {
            controller.send(Command::Connect {
                mode: Mode::Wifi,
                ip: host.clone(),
                http_port: *port,
                rtsp_port: config.rtsp_port,
            });
        }

        Self {
            controller,
            state,
            ip: config.ip.clone(),
            http_port: config.http_port.to_string(),
            rtsp_port: config.rtsp_port.to_string(),
            config,
            mode,
            preview_texture: None,
            show_log: false,
            last_sample: Instant::now(),
            last_frames: 0,
            last_sent: 0,
            last_dropped: 0,
            stream_fps: 0.0,
            sink_fps: 0.0,
            dropped_delta: 0,
            test_width,
            test_height,
            test_fps,
            palette: Palette::from_visuals(&egui::Visuals::dark()),
            theme_pref,
            preview_mode,
            sidebar_tab: 0,
        }
    }


    fn pump_preview(&mut self, ctx: &egui::Context) {
        if !self.config.preview_enabled {
            self.preview_texture = None;
            return;
        }
        let Some(frame) = self.state.preview.take() else {
            return;
        };
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [frame.width as usize, frame.height as usize],
            &frame.rgba,
        );
        if let Some(texture) = &mut self.preview_texture {
            texture.set(image, TextureOptions::LINEAR);
        } else {
            self.preview_texture =
                Some(ctx.load_texture("awc-preview", image, TextureOptions::LINEAR));
        }
    }

    fn tick_stats(&mut self, snapshot: &UiSnapshot) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_sample).as_secs_f32();
        if elapsed < 1.0 {
            return;
        }
        let frames = snapshot.stream_frames.saturating_sub(self.last_frames);
        let sent = snapshot.sink.sent.saturating_sub(self.last_sent);
        let dropped = snapshot.sink.dropped.saturating_sub(self.last_dropped);
        self.stream_fps = frames as f32 / elapsed;
        self.sink_fps = sent as f32 / elapsed;
        self.dropped_delta = dropped;
        self.last_frames = snapshot.stream_frames;
        self.last_sent = snapshot.sink.sent;
        self.last_dropped = snapshot.sink.dropped;
        self.last_sample = now;
    }

    fn top_bar(&mut self, ui: &mut egui::Ui, snapshot: &UiSnapshot, actions: &mut Vec<Command>) {
        let p = self.palette;
        egui::Panel::top("top")
            .show_separator_line(true)
            .frame(
                egui::Frame::default()
                    .fill(p.card)
                    .inner_margin(egui::Margin {
                        left: 16,
                        right: 16,
                        top: 8,
                        bottom: 8,
                    }),
            )
            .show_inside(ui, |ui| {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("AWC").size(20.0).strong().color(p.text));
                    ui.label(
                        RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .small()
                            .color(p.muted),
                    );
                    ui.separator();
                    let (status, variant) = status_badge(snapshot.conn.state);
                    ui.add(Badge::new(status).variant(variant));
                    ui.label(RichText::new(snapshot.conn.message.as_str()).color(p.text));
                    if let Some(protocol) = &snapshot.conn.protocol {
                        ui.add(Badge::new(protocol.as_str()).variant(BadgeVariant::Outline));
                    }

                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.set_min_width(ui.available_width());
                            if ui
                                .add(
                                    Button::new("Setup")
                                        .variant(ButtonVariant::Outline)
                                        .size(ButtonSize::Sm),
                                )
                                .clicked()
                            {
                                actions.push(Command::RunSetup);
                            }
                            if ui
                                .add(
                                    Button::new("Log")
                                        .variant(if self.show_log {
                                            ButtonVariant::Default
                                        } else {
                                            ButtonVariant::Ghost
                                        })
                                        .size(ButtonSize::Sm),
                                )
                                .clicked()
                            {
                                self.show_log = !self.show_log;
                            }

                            let mut pref = self.theme_pref;
                            let selected = theme_label(pref);
                            egui::ComboBox::from_id_salt("theme")
                                .selected_text(selected)
                                .width(82.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut pref,
                                        egui::ThemePreference::System,
                                        "System",
                                    );
                                    ui.selectable_value(
                                        &mut pref,
                                        egui::ThemePreference::Light,
                                        "Light",
                                    );
                                    ui.selectable_value(
                                        &mut pref,
                                        egui::ThemePreference::Dark,
                                        "Dark",
                                    );
                                });
                            if pref != self.theme_pref {
                                self.theme_pref = pref;
                                self.config.theme = theme_str(pref).to_string();
                                let _ = self.config.save();
                                ui.ctx().set_theme(pref);
                                self.palette = Palette::from_visuals(ui.visuals());
                            }

                            if !snapshot.registered {
                                ui.add(
                                    Badge::new("Not registered")
                                        .variant(BadgeVariant::Destructive),
                                );
                            }
                        },
                    );
                });
                ui.add_space(2.0);
            });
    }

    fn side_panel(&mut self, ui: &mut egui::Ui, snapshot: &UiSnapshot, actions: &mut Vec<Command>) {
        let panel = self.palette.sidebar;
        egui::Panel::left("controls")
            .show_separator_line(false)
            .resizable(false)
            .default_size(320.0)
            .frame(
                egui::Frame::default().fill(panel).inner_margin(egui::Margin {
                    left: 16,
                    right: 16,
                    top: 16,
                    bottom: 16,
                }),
            )
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    for (index, label) in ["Setup", "Camera", "Phone", "Test"].iter().enumerate() {
                        let active = self.sidebar_tab == index;
                        if ui
                            .add(
                                Button::new(*label)
                                    .variant(if active {
                                        ButtonVariant::Default
                                    } else {
                                        ButtonVariant::Ghost
                                    })
                                    .size(ButtonSize::Sm),
                            )
                            .clicked()
                        {
                            self.sidebar_tab = index;
                        }
                    }
                });
                ui.add_space(4.0);

                match self.sidebar_tab {
                    0 => Card::new().size(CardSize::Sm).show(ui, |ui| {
                        card_header(ui, "Connection", "USB or Wi-Fi");
                        card_content(ui, |ui| {
                            self.connection_ui(ui, snapshot, actions);
                        });
                    }),
                    1 => Card::new().size(CardSize::Sm).show(ui, |ui| {
                        card_header(ui, "Virtual camera", "DirectShow output");
                        card_content(ui, |ui| {
                            self.camera_ui(ui, snapshot, actions);
                        });
                    }),
                    2 => Card::new().size(CardSize::Sm).show(ui, |ui| {
                        card_header(ui, "Phone", "Camera controls");
                        card_content(ui, |ui| {
                            self.phone_ui(ui, snapshot, actions);
                        });
                    }),
                    _ => Card::new().size(CardSize::Sm).show(ui, |ui| {
                        card_header(ui, "Test source", "No phone required");
                        card_content(ui, |ui| {
                            self.test_source_ui(ui, snapshot, actions);
                        });
                    }),
                }
            });
    }

    fn connection_ui(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &UiSnapshot,
        actions: &mut Vec<Command>,
    ) {
        let p = self.palette;

        ui.horizontal(|ui| {
            let usb = Button::new("USB")
                .variant(if self.mode == Mode::Usb {
                    ButtonVariant::Default
                } else {
                    ButtonVariant::Outline
                })
                .size(ButtonSize::Sm);
            if ui.add(usb).clicked() {
                self.mode = Mode::Usb;
            }
            let wifi = Button::new("Wi-Fi")
                .variant(if self.mode == Mode::Wifi {
                    ButtonVariant::Default
                } else {
                    ButtonVariant::Outline
                })
                .size(ButtonSize::Sm);
            if ui.add(wifi).clicked() {
                self.mode = Mode::Wifi;
            }
        });

        match self.mode {
            Mode::Usb => {
                ui.label(RichText::new("Phone (ADB)").small().color(p.muted));
                let selected = snapshot
                    .selected_device
                    .clone()
                    .unwrap_or_else(|| "Select a device".to_string());
                egui::ComboBox::from_id_salt("device_combo")
                    .selected_text(selected)
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        for device in &snapshot.devices {
                            let label = format!("{} ({})", device.model, device.id);
                            let is_selected = snapshot
                                .selected_device
                                .as_deref()
                                .map(|id| id == device.id.as_str())
                                .unwrap_or(false);
                            if ui.selectable_label(is_selected, label).clicked() {
                                actions.push(Command::ConnectDevice(device.id.clone()));
                            }
                        }
                    });
                if ui
                    .add(
                        Button::new("Refresh devices")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Sm),
                    )
                    .clicked()
                {
                    actions.push(Command::RefreshDevices);
                }
            }
            Mode::Wifi => {
                ui.label(RichText::new("Phone IP").small().color(p.muted));
                ui.add(
                    Input::new(&mut self.ip)
                        .hint("192.168.1.100")
                        .max_width(ui.available_width()),
                );
                ui.horizontal(|ui| {
                    ui.label(RichText::new("HTTP").small().color(p.muted));
                    ui.add(Input::new(&mut self.http_port).max_width(64.0));
                    ui.label(RichText::new("RTSP").small().color(p.muted));
                    ui.add(Input::new(&mut self.rtsp_port).max_width(64.0));
                });
            }
        }

        ui.add_space(4.0);
        let connecting = snapshot.conn.state == ConnState::Connecting;
        let phone_live = snapshot.conn.state == ConnState::Streaming && !snapshot.test_running;
        let label = if connecting {
            "Connecting…"
        } else if phone_live {
            "Disconnect"
        } else if snapshot.test_running {
            "Connect phone"
        } else {
            "Connect"
        };
        let button = Button::new(label)
            .variant(if phone_live {
                ButtonVariant::Destructive
            } else {
                ButtonVariant::Default
            })
            .expanded(true)
            .enabled(!connecting);
        if ui.add(button).clicked() {
            if phone_live {
                actions.push(Command::Disconnect);
            } else {
                let http_port = self.http_port.trim().parse().unwrap_or(8080);
                let rtsp_port = self.rtsp_port.trim().parse().unwrap_or(8554);
                self.config.mode = match self.mode {
                    Mode::Usb => "usb".to_string(),
                    Mode::Wifi => "wifi".to_string(),
                };
                self.config.ip = self.ip.clone();
                self.config.http_port = http_port;
                self.config.rtsp_port = rtsp_port;
                let _ = self.config.save();
                actions.push(Command::Connect {
                    mode: self.mode,
                    ip: self.ip.clone(),
                    http_port,
                    rtsp_port,
                });
            }
        }

        let color = state_color(snapshot.conn.state, &p);
        ui.horizontal_wrapped(|ui| {
            status_dot(ui, color);
            ui.label(RichText::new(snapshot.conn.message.as_str()).small().color(p.muted));
        });
        if let Some(url) = &snapshot.conn.url {
            ui.label(RichText::new(url.as_str()).small().monospace().color(p.muted));
        }
    }

    fn test_source_ui(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &UiSnapshot,
        actions: &mut Vec<Command>,
    ) {
        let p = self.palette;
        let running = snapshot.test_running;
        let label = if running {
            "Stop test pattern"
        } else {
            "Start test pattern"
        };
        let button = Button::new(label)
            .variant(if running {
                ButtonVariant::Destructive
            } else {
                ButtonVariant::Outline
            })
            .expanded(true);
        if ui.add(button).clicked() {
            if running {
                actions.push(Command::StopTestPattern);
            } else {
                self.config.test_width = self.test_width;
                self.config.test_height = self.test_height;
                self.config.test_fps = self.test_fps;
                let _ = self.config.save();
                actions.push(Command::StartTestPattern {
                    width: self.test_width,
                    height: self.test_height,
                    fps: self.test_fps,
                });
            }
        }

        ui.label(
            RichText::new("Local colour bars; feeds the same preview + virtual-camera path.")
                .small()
                .color(p.muted),
        );

        ui.add_enabled_ui(!running, |ui| {
            ui.horizontal(|ui| {
                let current = test_label(self.test_width, self.test_height);
                egui::ComboBox::from_id_salt("test_res")
                    .selected_text(current)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        for (label, width, height) in TEST_PRESETS {
                            if ui.selectable_label(current == *label, *label).clicked() {
                                self.test_width = *width;
                                self.test_height = *height;
                            }
                        }
                    });

                let fps = format!("{} fps", self.test_fps);
                egui::ComboBox::from_id_salt("test_fps")
                    .selected_text(fps)
                    .width(90.0)
                    .show_ui(ui, |ui| {
                        for value in [30u32, 60] {
                            if ui
                                .selectable_label(self.test_fps == value, format!("{value} fps"))
                                .clicked()
                            {
                                self.test_fps = value;
                            }
                        }
                    });
            });
        });

        if running {
            ui.horizontal(|ui| {
                status_dot(ui, p.green);
                ui.label(
                    RichText::new(format!(
                        "Test pattern {}×{} @ {} fps",
                        self.test_width, self.test_height, self.test_fps
                    ))
                    .small()
                    .color(p.muted),
                );
            });
        }
    }

    fn camera_ui(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &UiSnapshot,
        actions: &mut Vec<Command>,
    ) {
        let p = self.palette;
        let running = snapshot.sink.running;
        let label = if running {
            "Stop virtual camera"
        } else {
            "Start virtual camera"
        };
        let button = Button::new(label)
            .variant(if running {
                ButtonVariant::Destructive
            } else {
                ButtonVariant::Default
            })
            .expanded(true);
        if ui.add(button).clicked() {
            actions.push(if running {
                Command::StopCam
            } else {
                Command::StartCam
            });
        }

        if let Some(error) = &snapshot.sink.error {
            ui.colored_label(p.red, error);
        }

        let consumer = if snapshot.sink.connected {
            ("In use by an app", p.green)
        } else if running {
            ("Waiting for Zoom / Teams / OBS", p.amber)
        } else {
            ("Not running", p.muted)
        };
        ui.horizontal(|ui| {
            status_dot(ui, consumer.1);
            ui.label(RichText::new(consumer.0).small().color(p.muted));
        });

        if running {
            let resolution = if snapshot.sink.width > 0 {
                format!("{}×{}", snapshot.sink.width, snapshot.sink.height)
            } else {
                "waiting for first frame".to_string()
            };
            ui.label(
                RichText::new(format!(
                    "Output {resolution} · frame #{} · source {:.1}s · output {:.1}s",
                    snapshot.sink.last_seq,
                    snapshot.stream_pts_ms as f32 / 1000.0,
                    snapshot.sink.last_pts_ms as f32 / 1000.0
                ))
                .small()
                .color(p.muted),
            );
            ui.horizontal(|ui| {
                metric(ui, "Sent", snapshot.sink.sent.to_string());
                metric(ui, "Dropped", snapshot.sink.dropped.to_string());
                metric(ui, "FPS", format!("{:.1}", self.sink_fps));
            });
        }

        ui.label(
            RichText::new(format!(
                "{} frames · {} errors · {} reconnects · {:.1} MB in · {} preview taps",
                snapshot.stream_frames,
                snapshot.stream_errors,
                snapshot.stream_reconnects,
                snapshot.stream_bytes as f64 / (1024.0 * 1024.0),
                snapshot.preview_taps
            ))
            .small()
            .color(p.muted),
        );
        if let Some(error) = &snapshot.stream_error {
            ui.colored_label(p.red, error);
        }
    }

    fn preview_controls_ui(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Command>) {
        let p = self.palette;
        let mut changed = false;

        ui.horizontal_wrapped(|ui| {
            let current_mode = preview_mode_label(self.preview_mode);
            egui::ComboBox::from_id_salt("preview_mode")
                .selected_text(current_mode)
                .width(190.0)
                .show_ui(ui, |ui| {
                    for (mode, label) in [
                        (PreviewMode::Throttled, "Throttled (lowest cost)"),
                        (PreviewMode::FullRate, "Full rate, downscaled"),
                        (PreviewMode::Native, "Native resolution + rate"),
                    ] {
                        if ui
                            .selectable_label(self.preview_mode == mode, label)
                            .clicked()
                        {
                            self.preview_mode = mode;
                            self.config.preview_mode = mode.as_str().to_string();
                            changed = true;
                        }
                    }
                });

            let throttled = self.preview_mode == PreviewMode::Throttled;
            changed |= ui
                .add_enabled(
                    throttled,
                    egui::Slider::new(&mut self.config.preview_fps, 1..=30)
                        .text("fps")
                        .max_decimals(0),
                )
                .changed();

            let native = self.preview_mode == PreviewMode::Native;
            ui.add_enabled_ui(!native, |ui| {
                let current = preview_label(self.config.preview_width, self.config.preview_height);
                egui::ComboBox::from_id_salt("preview_res")
                    .selected_text(current)
                    .width(170.0)
                    .show_ui(ui, |ui| {
                        for (label, width, height) in PREVIEW_PRESETS {
                            if ui.selectable_label(current == *label, *label).clicked() {
                                self.config.preview_width = *width;
                                self.config.preview_height = *height;
                                changed = true;
                            }
                        }
                    });
            });
        });

        if changed {
            let config = self.config.preview();
            let _ = self.config.save();
            actions.push(Command::UpdatePreview(config));
        }

        let stats = self.state.preview.stats();
        ui.label(
            RichText::new(format!(
                "Preview work {:.2} ms/frame (last {:.2} ms) · {} taps · decode thread",
                stats.average_micros() / 1000.0,
                stats.last_micros as f64 / 1000.0,
                stats.taps
            ))
            .small()
            .color(p.muted),
        );
        let hint = match self.preview_mode {
            PreviewMode::Throttled => {
                "Downsample + fps cap. Cheapest; the decode thread does little extra work."
            }
            PreviewMode::FullRate => {
                "Downsample every decoded frame. More decode-thread work, no fps cap."
            }
            PreviewMode::Native => {
                "Full resolution every frame. Most expensive; can reduce main-stream fps."
            }
        };
        ui.label(RichText::new(hint).small().color(p.muted));
    }

    fn phone_ui(&mut self, ui: &mut egui::Ui, snapshot: &UiSnapshot, actions: &mut Vec<Command>) {
        let p = self.palette;
        let features = self.state.features.lock().unwrap().clone();
        let Some(features) = features.as_ref() else {
            ui.label(RichText::new("Connect to load camera controls.").small().color(p.muted));
            return;
        };

        ui.label(
            RichText::new(format!("Protocol: {}", features.stream_protocol))
                .small()
                .color(p.muted),
        );

        if !features.resolutions.is_empty() {
            let current = snapshot
                .selected_resolution
                .clone()
                .unwrap_or_else(|| "Resolution".to_string());
            egui::ComboBox::from_id_salt("resolution")
                .selected_text(current.clone())
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    for resolution in &features.resolutions {
                        if ui
                            .selectable_label(&current == resolution, resolution)
                            .clicked()
                        {
                            actions.push(Command::SetResolution(resolution.clone()));
                        }
                    }
                });
        }

        ui.horizontal(|ui| {
            if ui.button("Switch camera").clicked() {
                actions.push(Command::Control("switch_camera=true".to_string()));
            }
            if features.manual_focus {
                ui.label(RichText::new("Manual focus available").small().color(p.muted));
            }
        });

        if features.has_zoom {
            ui.label(
                RichText::new(format!(
                    "Zoom range: {:.1}× – {:.1}×",
                    features.zoom_min, features.zoom_max
                ))
                .small()
                .color(p.muted),
            );
        }
        if features.exposure_upper > features.exposure_lower {
            ui.label(
                RichText::new(format!(
                    "Exposure range: {} – {}",
                    features.exposure_lower, features.exposure_upper
                ))
                .small()
                .color(p.muted),
            );
        }
    }

    fn central(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Command>) {
        let bg = self.palette.card;
        let card = self.palette.card;
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(bg))
            .show_inside(ui, |ui| {
                egui::Frame::default()
                    .fill(card)
                    .inner_margin(egui::Margin {
                        left: 16,
                        right: 16,
                        top: 0,
                        bottom: 16,
                    })
                    .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (status, variant) = status_badge(self.state.conn.lock().unwrap().state);
                        ui.add(Badge::new(status).variant(variant));
                        ui.label(
                            RichText::new(format!(
                                "{:.1} fps stream · {:.1} fps output",
                                self.stream_fps, self.sink_fps
                            ))
                            .small()
                            .color(self.palette.muted),
                        );

                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                let mut enabled = self.config.preview_enabled;
                                if ui.add(Switch::new(&mut enabled)).changed() {
                                    self.config.preview_enabled = enabled;
                                    let config = self.config.preview();
                                    let _ = self.config.save();
                                    actions.push(Command::UpdatePreview(config));
                                }
                                ui.label(
                                    RichText::new("Preview")
                                        .small()
                                        .color(self.palette.muted),
                                );
                            },
                        );
                    });

                    if self.config.preview_enabled {
                        ui.add_space(2.0);
                        self.preview_controls_ui(ui, actions);
                    }
                    ui.add_space(8.0);
                    let min_height = (ui.available_height() - 16.0).max(220.0);
                    ui.set_min_height(min_height);
                    if !self.config.preview_enabled {
                        placeholder(ui, "Preview off", "Turn on Preview above.");
                        return;
                    }
                    let Some(texture) = &self.preview_texture else {
                        placeholder(
                            ui,
                            "No signal",
                            "Connect to the phone and start the stream.",
                        );
                        return;
                    };
                    let rect = ui.available_rect_before_wrap().shrink(16.0);
                    let size = fit_size(rect.size(), texture.size_vec2());
                    let image_rect = egui::Rect::from_center_size(rect.center(), size);
                    let radius = egui::CornerRadius::same(8);
                    let shadow = egui::epaint::Shadow {
                        offset: [0, 3],
                        blur: 16,
                        spread: 2,
                        color: Color32::from_black_alpha(if ui.visuals().dark_mode {
                            80
                        } else {
                            24
                        }),
                    };
                    ui.painter().add(shadow.as_shape(image_rect, radius));
                    ui.put(
                        image_rect,
                        egui::Image::new(egui::load::SizedTexture::new(texture.id(), size))
                            .corner_radius(radius),
                    );
                    });
            });
    }

    fn log_panel(&mut self, ui: &mut egui::Ui) {
        if !self.show_log {
            return;
        }
        let p = self.palette;
        egui::Panel::bottom("log")
            .resizable(true)
            .default_size(130.0)
            .frame(
                egui::Frame::default()
                    .fill(p.card)
                    .inner_margin(egui::Margin::symmetric(16, 12)),
            )
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("LOG").small().strong().color(p.muted));
                    if ui
                        .add(
                            Button::new("Clear")
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm),
                        )
                        .clicked()
                    {
                        self.state.logs.lock().unwrap().clear();
                    }
                    ui.label(
                        RichText::new(format!(
                            "{} fps in · {} fps out · {} dropped/s",
                            self.stream_fps, self.sink_fps, self.dropped_delta
                        ))
                        .small()
                        .color(p.muted),
                    );
                });
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let logs = self.state.logs.lock().unwrap();
                        for line in logs.iter() {
                            let color = match line.level {
                                LogLevel::Info => p.muted,
                                LogLevel::Warn => p.amber,
                                LogLevel::Error => p.red,
                            };
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("{:>6.1}s", line.at.elapsed().as_secs_f32()))
                                        .monospace()
                                        .small()
                                        .color(p.muted),
                                );
                                ui.label(RichText::new(line.text.as_str()).small().color(color));
                            });
                        }
                    });
            });
    }

}

impl eframe::App for AwcApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.palette = Palette::from_visuals(ui.visuals());
        let snapshot = self.state.snapshot();
        self.pump_preview(&ctx);
        self.tick_stats(&snapshot);

        let mut actions: Vec<Command> = Vec::new();
        let canvas = self.palette.canvas;
        let border = self.palette.border;
        ui.style_mut().visuals.widgets.noninteractive.bg_stroke =
            egui::Stroke::new(1.0_f32, border);
        egui::Frame::default()
            .fill(canvas)
            .inner_margin(egui::Margin::same(0))
            .show(ui, |ui| {
                self.top_bar(ui, &snapshot, &mut actions);
                self.log_panel(ui);
                self.side_panel(ui, &snapshot, &mut actions);
                self.central(ui, &mut actions);
            });

        for command in actions {
            self.controller.send(command);
        }

        let interval = if self.config.preview_enabled {
            33
        } else {
            200
        };
        ctx.request_repaint_after(Duration::from_millis(interval));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.controller.shutdown();
    }
}

fn metric(ui: &mut egui::Ui, label: &str, value: String) {
    ui.vertical(|ui| {
        ui.label(RichText::new(value).strong().color(ui.visuals().text_color()));
        ui.label(
            RichText::new(label)
                .small()
                .color(ui.visuals().weak_text_color()),
        );
    });
    ui.add_space(12.0);
}

fn state_color(state: ConnState, palette: &Palette) -> Color32 {
    match state {
        ConnState::Idle => palette.muted,
        ConnState::Connecting => palette.amber,
        ConnState::Streaming => palette.green,
        ConnState::Error => palette.red,
    }
}

fn status_badge(state: ConnState) -> (&'static str, BadgeVariant) {
    match state {
        ConnState::Idle => ("Idle", BadgeVariant::Secondary),
        ConnState::Connecting => ("Connecting", BadgeVariant::Outline),
        ConnState::Streaming => ("Live", BadgeVariant::Default),
        ConnState::Error => ("Error", BadgeVariant::Destructive),
    }
}

fn preview_mode_label(mode: PreviewMode) -> &'static str {
    match mode {
        PreviewMode::Throttled => "Throttled",
        PreviewMode::FullRate => "Full rate (downscaled)",
        PreviewMode::Native => "Native (full quality)",
    }
}

fn theme_label(pref: egui::ThemePreference) -> &'static str {
    match pref {
        egui::ThemePreference::System => "System",
        egui::ThemePreference::Light => "Light",
        egui::ThemePreference::Dark => "Dark",
    }
}

fn theme_str(pref: egui::ThemePreference) -> &'static str {
    match pref {
        egui::ThemePreference::System => "system",
        egui::ThemePreference::Light => "light",
        egui::ThemePreference::Dark => "dark",
    }
}

fn preview_label(width: u32, height: u32) -> &'static str {
    PREVIEW_PRESETS
        .iter()
        .find(|(_, w, h)| *w == width && *h == height)
        .map(|(label, _, _)| *label)
        .unwrap_or("custom")
}

fn test_label(width: u32, height: u32) -> &'static str {
    TEST_PRESETS
        .iter()
        .find(|(_, w, h)| *w == width && *h == height)
        .map(|(label, _, _)| *label)
        .unwrap_or("custom")
}

fn fit_size(available: egui::Vec2, texture: egui::Vec2) -> egui::Vec2 {
    if available.x <= 1.0 || available.y <= 1.0 || texture.x <= 1.0 || texture.y <= 1.0 {
        return egui::vec2(1.0, 1.0);
    }
    let aspect = texture.x / texture.y;
    let mut size = available;
    if size.x / size.y > aspect {
        size.x = size.y * aspect;
    } else {
        size.y = size.x / aspect;
    }
    size
}

fn placeholder(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    let rect = ui.available_rect_before_wrap();
    let text = ui.visuals().text_color();
    let muted = ui.visuals().weak_text_color();
    let painter = ui.painter();
    painter.text(
        rect.center() - egui::vec2(0.0, 12.0),
        egui::Align2::CENTER_CENTER,
        title,
        egui::FontId::proportional(24.0),
        text,
    );
    painter.text(
        rect.center() + egui::vec2(0.0, 18.0),
        egui::Align2::CENTER_CENTER,
        subtitle,
        egui::FontId::proportional(12.0),
        muted,
    );
}

fn status_dot(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
}
