//! Virtual camera registration / setup checks.
//!
//! The registration mechanism is intentionally unchanged: `regsvr32` on
//! `softcam.dll`, with the same CLSID check the original installer used.

use std::path::PathBuf;
use std::process::Command;

use crate::core::state::SharedState;

fn softcam_dll() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("softcam.dll"));
        }
    }
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../CLIENT/tauri-client/src-tauri/softcam.dll"),
    );
    candidates.push(PathBuf::from("softcam.dll"));
    candidates.into_iter().find(|path| path.exists())
}

pub fn is_registered() -> bool {
    let output = Command::new("reg")
        .args([
            "query",
            r"HKCR\CLSID\{860BB310-5D01-11d0-BD3B-00A0C911CE86}\Instance",
            "/s",
        ])
        .output();
    match output {
        Ok(output) => String::from_utf8_lossy(&output.stdout).contains("AWC Virtual Cam"),
        Err(_) => false,
    }
}

fn msvc_runtime_present() -> bool {
    let output = Command::new("reg")
        .args([
            "query",
            r"HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64",
            "/v",
            "Version",
        ])
        .output();
    let Ok(output) = output else {
        return false;
    };
    let text = String::from_utf8_lossy(&output.stdout);
    text.split_whitespace()
        .find(|token| token.starts_with('v'))
        .and_then(|version| {
            version
                .trim_start_matches('v')
                .split('.')
                .next()
                .and_then(|major| major.parse::<u32>().ok())
        })
        .map(|major| major >= 14)
        .unwrap_or(false)
}

fn register(dll: &PathBuf) -> Result<(), String> {
    let dll_str = dll.to_string_lossy().to_string();
    let status = Command::new("regsvr32")
        .args(["/s", dll_str.as_str()])
        .status()
        .map_err(|e| format!("regsvr32: {e}"))?;
    if status.success() {
        return Ok(());
    }

    // Retry once with elevation; this is exactly what the original installer
    // did, and it keeps the registration flow unchanged.
    let script = format!(
        "Start-Process regsvr32 -ArgumentList '/s \"{}\"' -Verb RunAs -Wait",
        dll.display()
    );
    let status = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .status()
        .map_err(|e| format!("powershell regsvr32: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("registration failed (try running as administrator)".to_string())
    }
}

/// Run the setup steps synchronously.  Call from the controller thread; the
/// UI keeps painting and reads the log as it goes.
pub fn run_setup(state: &SharedState) {
    state.info("Setup: checking required files...");
    let Some(dll) = softcam_dll() else {
        state.error("Setup: softcam.dll not found next to the executable");
        return;
    };
    state.info(format!("Setup: softcam.dll at {}", dll.display()));

    if is_registered() {
        state.info("Setup: AWC Virtual Cam is already registered");
    } else {
        state.warn("Setup: AWC Virtual Cam is not registered; registering...");
        match register(&dll) {
            Ok(()) => {
                if is_registered() {
                    state.info("Setup: virtual camera registered");
                } else {
                    state.warn("Setup: regsvr32 succeeded but the CLSID was not found");
                }
            }
            Err(err) => state.error(format!("Setup: {err}")),
        }
    }

    if msvc_runtime_present() {
        state.info("Setup: MSVC runtime present");
    } else {
        state.warn("Setup: MSVC runtime not detected (install vc_redist.x64 if the app fails to start)");
    }
}
