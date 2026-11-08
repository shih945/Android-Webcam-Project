//! ADB device discovery and USB port forwarding.
//!
//! The commands are blocking; callers run them on the controller thread, never
//! on the egui thread.

use std::path::PathBuf;
use std::process::Command;

use crate::core::state::Device;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn adb_path() -> Result<PathBuf, String> {
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("adb/adb.exe"));
        }
    }
    // Development convenience: reuse the ADB bundled with the Tauri client.
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../CLIENT/tauri-client/src-tauri/adb/adb.exe"),
    );
    candidates.push(PathBuf::from("adb/adb.exe"));
    candidates.push(PathBuf::from("adb.exe"));

    for path in &candidates {
        if path.exists() {
            return Ok(path.clone());
        }
    }
    Err("adb.exe not found (expected next to the app in adb/adb.exe)".to_string())
}

fn command(adb: &PathBuf) -> Command {
    let mut command = Command::new(adb.as_os_str());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

pub fn list_devices() -> Result<Vec<Device>, String> {
    let adb = adb_path()?;
    let output = command(&adb)
        .arg("devices")
        .output()
        .map_err(|e| format!("adb devices: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "adb devices failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    for line in text.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('*') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(serial) = parts.next() else {
            continue;
        };
        let state = parts.next().unwrap_or("");
        if state != "device" {
            continue;
        }
        let model = command(&adb)
            .args(["-s", serial, "shell", "getprop", "ro.product.model"])
            .output()
            .ok()
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .filter(|model| !model.is_empty())
            .unwrap_or_else(|| serial.to_string());
        devices.push(Device {
            id: serial.to_string(),
            model,
        });
    }
    Ok(devices)
}

pub fn forward_device(device_id: &str) -> Result<(), String> {
    let adb = adb_path()?;
    for port in ["8080", "8554"] {
        let forward = format!("tcp:{port}");
        let output = command(&adb)
            .args(["-s", device_id, "forward", forward.as_str(), forward.as_str()])
            .output()
            .map_err(|e| format!("adb forward tcp:{port}: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "adb forward tcp:{port} failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
    }
    Ok(())
}
