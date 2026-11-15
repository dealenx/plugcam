pub mod adb;
pub mod app;
#[cfg(windows)]
#[path = "decode.rs"]
pub mod decode;
#[cfg(unix)]
#[path = "decode_unix.rs"]
pub mod decode;
pub mod diag;
pub mod frame;
pub mod pipeline;
#[cfg(windows)]
#[path = "platform.rs"]
pub mod platform;
#[cfg(unix)]
#[path = "platform_unix.rs"]
pub mod platform;
pub mod portable;
pub mod preview;
pub mod resources;
pub mod scrcpy;
pub mod settings;
#[cfg(windows)]
#[path = "vcam.rs"]
pub mod vcam;
#[cfg(unix)]
#[path = "vcam_unix.rs"]
pub mod vcam;
pub mod wifi;
