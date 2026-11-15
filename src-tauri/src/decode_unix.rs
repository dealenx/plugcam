//! H.264 → NV12 with the OpenH264 decoder (Cisco), on the CPU.
//!
//! scrcpy feeds complete Annex B access units (or SPS/PPS), the same as on Windows.
//! OpenH264 hands out planar YUV420; the frame converter wants NV12, so U and V are
//! interleaved per row. Colorimetry is not in the bitstream here: phones stream
//! full-range BT.601, which is `ColorInfo::default()` too (see `crate::frame`).

use openh264::decoder::Decoder;
use openh264::formats::YUVSource;

use crate::frame::{ColorInfo, Nv12Frame};

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("OpenH264: {0}")]
    Codec(#[from] openh264::Error),
}

pub struct H264Decoder {
    decoder: Decoder,
    /// Interleaved UV plane of one picture, rebuilt on every frame.
    uv: Vec<u8>,
}

impl H264Decoder {
    /// `width`/`height` come from the scrcpy session packet; OpenH264 reads the real
    /// geometry from the SPS, so they are only a hint here.
    pub fn new(_width: u32, _height: u32) -> Result<Self, DecodeError> {
        Ok(Self { decoder: Decoder::new()?, uv: Vec::new() })
    }

    /// OpenH264 decodes on the CPU only.
    pub fn is_hardware(&self) -> bool {
        false
    }

    /// Feeds one Annex B access unit (a frame, or SPS/PPS) and calls `on_frame` for every
    /// picture that comes out.
    pub fn decode(&mut self, data: &[u8], _pts_us: u64, on_frame: &mut dyn FnMut(&Nv12Frame)) -> Result<(), DecodeError> {
        // Destructure so `decoder` and `uv` borrow disjoint halves of `self`.
        let H264Decoder { decoder, uv } = self;
        for nal in openh264::nal_units(data) {
            let Some(yuv) = decoder.decode(nal)? else { continue };
            let (width, height) = yuv.dimensions();
            let (y_stride, u_stride, v_stride) = yuv.strides();
            let half_w = width / 2;
            let half_h = height / 2;
            let uv_stride = width; // NV12: 2 bytes per pixel, half the rows

            uv.clear();
            uv.reserve(uv_stride * half_h);
            for row in 0..half_h {
                let u = &yuv.u()[row * u_stride..row * u_stride + half_w];
                let v = &yuv.v()[row * v_stride..row * v_stride + half_w];
                for i in 0..half_w {
                    uv.push(u[i]);
                    uv.push(v[i]);
                }
            }

            on_frame(&Nv12Frame {
                width: width as u32,
                height: height as u32,
                y: yuv.y(),
                y_stride: y_stride as u32,
                uv,
                uv_stride: uv_stride as u32,
                color: ColorInfo::default(),
            });
        }
        Ok(())
    }
}
