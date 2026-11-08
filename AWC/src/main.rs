#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod core;
mod video;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let autostart_test = args.iter().any(|arg| arg == "--test-pattern");
    let autostart_cam = args.iter().any(|arg| arg == "--start-cam");
    let no_preview = args.iter().any(|arg| arg == "--no-preview");

    let mut wifi: Option<(String, u16)> = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--wifi" => {
                if let Some(value) = args.get(index + 1) {
                    let (host, port) = match value.split_once(':') {
                        Some((host, port)) => {
                            (host.to_string(), port.parse().unwrap_or(8080))
                        }
                        None => (value.to_string(), 8080),
                    };
                    wifi = Some((host, port));
                    index += 1;
                }
            }
            "--http-port" => {
                if let Some(value) = args.get(index + 1) {
                    let port = value.parse().unwrap_or(8080);
                    match wifi.take() {
                        Some((host, _)) => wifi = Some((host, port)),
                        None => wifi = Some(("127.0.0.1".to_string(), port)),
                    }
                    index += 1;
                }
            }
            _ => {}
        }
        index += 1;
    }

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("AWC")
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([700.0, 440.0])
            .with_maximized(true),
        ..Default::default()
    };

    eframe::run_native(
        "AWC",
        options,
        Box::new(move |cc| {
            Ok(Box::new(app::AwcApp::new(
                cc,
                autostart_test,
                autostart_cam,
                no_preview,
                wifi.clone(),
            )))
        }),
    )
}
