//! Phone HTTP endpoints (`/features`, `/control`).
//!
//! These are short request/response calls, so a total timeout is appropriate
//! here (unlike the long-lived MJPEG request in `video::source`).

use std::time::Duration;

use serde_json::Value;

use crate::core::state::Features;

fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(4))
        .build()
        .map_err(|e| format!("HTTP client: {e}"))
}

pub fn fetch_features(host: &str, port: u16) -> Result<Features, String> {
    let url = format!("http://{host}:{port}/features");
    let response = client()?
        .get(&url)
        .send()
        .map_err(|e| format!("GET {url}: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("GET {url}: HTTP {}", response.status()));
    }
    let value: Value = response
        .json()
        .map_err(|e| format!("GET {url}: invalid JSON: {e}"))?;

    let resolutions = value
        .get("resolutions")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let protocol = value
        .get("stream_protocol")
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .unwrap_or_else(|| v.to_string())
        })
        .unwrap_or_else(|| "MJPEG".to_string());

    let rtsp_port = value
        .get("rtsp_port")
        .and_then(Value::as_u64)
        .or_else(|| value.get("stream_port").and_then(Value::as_u64))
        .filter(|port| *port > 0 && *port < 65536)
        .map(|port| port as u16);

    Ok(Features {
        resolutions,
        stream_protocol: protocol,
        rtsp_port,
        manual_focus: value
            .get("manual_focus")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        has_zoom: value
            .get("has_zoom")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        zoom_min: value
            .get("zoom_min")
            .and_then(Value::as_f64)
            .unwrap_or(1.0) as f32,
        zoom_max: value
            .get("zoom_max")
            .and_then(Value::as_f64)
            .unwrap_or(1.0) as f32,
        exposure_lower: value
            .get("exposure_lower")
            .and_then(Value::as_i64)
            .unwrap_or(0) as i32,
        exposure_upper: value
            .get("exposure_upper")
            .and_then(Value::as_i64)
            .unwrap_or(0) as i32,
    })
}

pub fn is_rtsp(features: &Features) -> bool {
    let protocol = features.stream_protocol.to_ascii_uppercase();
    protocol.contains("RTSP") || protocol.contains("H264")
}

#[derive(Clone, Debug)]
pub struct RemoteSettings {
    pub camera: String,
}

/// Current camera-facing and resolution reported by `/settings`.
pub fn fetch_settings(host: &str, port: u16) -> Result<RemoteSettings, String> {
    let url = format!("http://{host}:{port}/settings");
    let response = client()?
        .get(&url)
        .send()
        .map_err(|e| format!("GET {url}: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("GET {url}: HTTP {}", response.status()));
    }
    let value: Value = response
        .json()
        .map_err(|e| format!("GET {url}: invalid JSON: {e}"))?;
    Ok(RemoteSettings {
        camera: value
            .get("camera")
            .and_then(Value::as_str)
            .unwrap_or("back")
            .to_string(),
    })
}

pub fn send_control(host: &str, port: u16, query: &str) -> Result<(), String> {
    let url = format!("http://{host}:{port}/control?{query}");
    let response = client()?
        .get(&url)
        .send()
        .map_err(|e| format!("GET {url}: {e}"))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("GET {url}: HTTP {}", response.status()))
    }
}
