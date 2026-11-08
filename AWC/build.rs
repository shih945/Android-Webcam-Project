//! Copy the runtime pieces that cannot be linked on the GNU toolchain:
//! `softcam.dll` (loaded dynamically) and the FFmpeg shared DLLs.  ADB is
//! copied too so USB mode works from the build output without extra setup.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let resources = manifest.join("../CLIENT/tauri-client/src-tauri");
    let softcam = resources.join("softcam.dll");
    println!("cargo:rerun-if-changed={}", softcam.display());

    let Some(profile_dir) = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .and_then(|out| out.ancestors().nth(3).map(Path::to_path_buf))
    else {
        return;
    };

    copy_if_present(&softcam, &profile_dir.join("softcam.dll"));

    if let Some(ffmpeg_dir) = env::var_os("FFMPEG_DIR") {
        let bin = PathBuf::from(ffmpeg_dir).join("bin");
        if let Ok(entries) = fs::read_dir(&bin) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("dll") {
                    if let Some(name) = path.file_name() {
                        copy_if_present(&path, &profile_dir.join(name));
                    }
                }
            }
        }
    }

    let adb_src = resources.join("adb");
    let adb_dst = profile_dir.join("adb");
    if adb_src.is_dir() {
        let _ = fs::create_dir_all(&adb_dst);
        if let Ok(entries) = fs::read_dir(&adb_src) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name() {
                        copy_if_present(&path, &adb_dst.join(name));
                    }
                }
            }
        }
    }
}

fn copy_if_present(src: &Path, dst: &Path) {
    if !src.exists() {
        return;
    }
    if let Ok(meta) = fs::metadata(dst) {
        if let Ok(src_meta) = fs::metadata(src) {
            if meta.len() == src_meta.len() {
                return;
            }
        }
    }
    let _ = fs::copy(src, dst);
}
