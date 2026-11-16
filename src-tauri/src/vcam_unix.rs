//! "Plugcam Camera": the v4l2loopback virtual camera on Linux.
//!
//! Frames are written as BGR3 into a loopback device created by the `v4l2loopback`
//! kernel module. There is no registration step like Windows' regsvr32: the module must
//! be loaded (`sudo modprobe v4l2loopback`), and the device comes from
//! `PLUGCAM_VCAM_DEVICE`, else the first loopback card, else `/dev/video2`.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use v4l::video::Output;
use v4l::{Device, Format, FourCC};

#[derive(Debug, thiserror::Error)]
pub enum VcamError {
    #[error("cannot open {path}: {source}")]
    Open { path: String, source: std::io::Error },
    #[error("width and height must be positive multiples of 4, got {0}x{1}")]
    BadSize(u32, u32),
    #[error(
        "{path} does not accept {width}x{height} BGR3 — another app (e.g. OBS) has the \
         camera open with a different size; close it or match {width}x{height} in its settings"
    )]
    Format { path: String, width: u32, height: u32 },
}

/// A live virtual camera on a v4l2loopback device. Frames are 24-bit BGR, rows top to
/// bottom, no padding.
pub struct VirtualCamera {
    file: std::fs::File,
    path: PathBuf,
    width: u32,
    height: u32,
}

// Written only from the pipeline thread, the same contract as the Windows sender API.
unsafe impl Send for VirtualCamera {}

impl VirtualCamera {
    /// Opens the loopback device and sets `width`x`height` BGR3. The `dll` parameter is
    /// the Windows camera DLL; on Linux the device comes from `device_path()` instead.
    pub fn create(_dll: &Path, width: u32, height: u32) -> Result<Self, VcamError> {
        if width == 0 || height == 0 || width % 4 != 0 || height % 4 != 0 {
            return Err(VcamError::BadSize(width, height));
        }
        let path = device_path();
        let open_err = |source| VcamError::Open { path: path.display().to_string(), source };

        let dev = Device::with_path(&path).map_err(open_err)?;
        let want = Format::new(width, height, FourCC::new(b"BGR3"));
        let got = dev.set_format(&want).map_err(open_err)?;
        if (got.width, got.height, got.fourcc) != (width, height, want.fourcc) {
            return Err(VcamError::Format { path: path.display().to_string(), width, height });
        }

        let file = OpenOptions::new().write(true).open(&path).map_err(open_err)?;
        Ok(Self { file, path, width, height })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Byte length `send_frame` expects.
    pub fn frame_len(&self) -> usize {
        self.width as usize * self.height as usize * 3
    }

    /// Publishes one BGR frame. Panics if the buffer size does not match the camera.
    pub fn send_frame(&self, bgr: &[u8]) {
        assert_eq!(bgr.len(), self.frame_len(), "frame size does not match the camera");
        // A short write would tear the picture; v4l2loopback takes whole frames.
        (&self.file).write_all(bgr).unwrap_or_else(|e| panic!("{}: {e}", self.path.display()));
    }

    /// v4l2loopback exposes no connection counter through V4L2; report "open" always so
    /// the UI does not flicker. (shortcut: connection state is unknown on Linux)
    pub fn is_connected(&self) -> bool {
        true
    }
}

/// `PLUGCAM_VCAM_DEVICE`, else the first v4l2loopback device, else `/dev/video2`.
pub fn device_path() -> PathBuf {
    if let Some(p) = std::env::var_os("PLUGCAM_VCAM_DEVICE") {
        return PathBuf::from(p);
    }
    (0..16).map(|i| PathBuf::from(format!("/dev/video{i}"))).find(|p| is_loopback(p)).unwrap_or_else(|| PathBuf::from("/dev/video2"))
}

fn is_loopback(path: &Path) -> bool {
    Device::with_path(path).is_ok_and(|d| d.query_caps().is_ok_and(|c| c.driver == "v4l2 loopback"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_sizes_softcam_cannot_handle() {
        for (w, h) in [(0, 1080), (1920, 0), (1922, 1080), (1920, 1082)] {
            // Fails on size before touching any device.
            let err = VirtualCamera::create(Path::new("unused"), w, h).err().unwrap();
            assert!(matches!(err, VcamError::BadSize(..)), "{w}x{h}: {err}");
        }
    }
}
