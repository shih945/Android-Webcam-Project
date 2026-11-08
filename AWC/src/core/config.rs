//! Small JSON config stored next to the user's application data.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::video::{PreviewConfig, PreviewMode};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub mode: String,
    pub ip: String,
    pub http_port: u16,
    pub rtsp_port: u16,
    pub hw_decode: bool,
    pub preview_enabled: bool,
    pub preview_mode: String,
    pub preview_fps: u32,
    pub preview_width: u32,
    pub preview_height: u32,
    pub theme: String,
    pub language: String,
    pub rotation: u16,
    pub mirror: bool,
    pub test_width: u32,
    pub test_height: u32,
    pub test_fps: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: "usb".to_string(),
            ip: "192.168.1.100".to_string(),
            http_port: 8080,
            rtsp_port: 8554,
            hw_decode: true,
            preview_enabled: true,
            preview_mode: "throttled".to_string(),
            preview_fps: 8,
            preview_width: 640,
            preview_height: 360,
            theme: "system".to_string(),
            language: "system".to_string(),
            rotation: 0,
            mirror: false,
            test_width: 1280,
            test_height: 720,
            test_fps: 30,
        }
    }
}

impl Config {
    pub fn preview(&self) -> PreviewConfig {
        PreviewConfig {
            enabled: self.preview_enabled,
            mode: PreviewMode::parse(&self.preview_mode),
            fps: self.preview_fps.clamp(1, 30),
            max_width: self.preview_width.clamp(160, 1920),
            max_height: self.preview_height.clamp(90, 1080),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = config_path().ok_or("no config directory")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        let text =
            serde_json::to_string_pretty(self).map_err(|e| format!("serialize config: {e}"))?;
        fs::write(&path, text).map_err(|e| format!("write {}: {e}", path.display()))
    }
}

pub fn load() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    let Ok(text) = fs::read_to_string(&path) else {
        return Config::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

fn config_path() -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())?;
    Some(base.join("AWC").join("config.json"))
}
