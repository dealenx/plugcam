//! The system facts the app needs on Linux: what to show about the PC, the Downloads
//! folder, and whether the virtual camera device is present.

use std::path::PathBuf;

/// What the status shows when the virtual camera is missing.
pub const VCAM_PROBLEM: &str = "Plugcam Camera needs the v4l2loopback module (sudo modprobe v4l2loopback exclusive_caps=1)";
/// What a bug report calls the virtual camera.
pub const VCAM_LABEL: &str = "Camera device";

/// e.g. `Linux 6.12.9-1-cachyos (x86_64)`.
pub fn os_name() -> String {
    let release = std::fs::read_to_string("/proc/sys/kernel/osrelease").map(|s| s.trim().to_string()).unwrap_or_default();
    format!("Linux {release} ({})", std::env::consts::ARCH)
}

/// No per-OS build number to compare against; 0 keeps the mica check off.
pub fn os_build() -> u32 {
    0
}

/// Graphics adapter names. (shortcut: read from sysfs on demand, empty for now)
pub fn gpu_names() -> Vec<String> {
    Vec::new()
}

/// The user's Downloads folder.
pub fn downloads_dir() -> Option<PathBuf> {
    let dir = PathBuf::from(std::env::var_os("HOME")?).join("Downloads");
    dir.is_dir().then_some(dir)
}

/// Path of the loopback device feeding "Plugcam Camera", if the module is loaded.
pub fn vcam_registered_path() -> Option<String> {
    let path = crate::vcam::device_path();
    // is_file() is false for /dev/* character devices; presence is what we need.
    path.exists().then(|| path.display().to_string())
}
