//! Safe wrapper around SVT-AV1 encoder

use mead_core::{ArcFrame, PixelFormat, Error, Result};
use mead_core::codec::VideoEncoder;
use std::ptr;
use svt_av1_sys::*;

/// Configuration for SVT-AV1 encoder
#[derive(Debug, Clone)]
pub struct SvtAv1Config {
    /// Encoder preset (0=best quality/slowest, 13=fastest/lower quality)
    pub preset: u8,

    /// Width in pixels
    pub width: u32,

    /// Height in pixels
    pub height: u32,

    /// Frame rate numerator
    pub fps_num: u32,

    /// Frame rate denominator
    pub fps_den: u32,

    /// Quantization parameter (0-63, lower=better quality)
    /// Only used in CRF mode
    pub qp: u32,

    /// Bit depth (8 or 10)
    pub bit_depth: u32,

    /// Number of tile columns (0 = auto)
    pub tile_cols: i32,

    /// Number of tile rows (0 = auto)
    pub tile_rows: i32,
}

impl Default for SvtAv1Config {
    fn default() -> Self {
        Self {
            preset: 8,        // Balanced preset
            width: 0,         // Must be set
            height: 0,        // Must be set
            fps_num: 30,
            fps_den: 1,
            qp: 35,          // Reasonable quality
            bit_depth: 8,
            tile_cols: 0,    // Auto
            tile_rows: 0,    // Auto
        }
    }
}

/// Safe wrapper around SVT-AV1 encoder
pub struct SvtAv1Encoder {
    handle: *mut EbComponentType,
    width: u32,
    height: u32,
    frame_count: u64,
}

impl SvtAv1Encoder {
    /// Create new encoder with configuration
    pub fn new(config: SvtAv1Config) -> Result<Self> {
        if config.width == 0 || config.height == 0 {
            return Err(Error::InvalidInput("Width and height must be set".to_string()));
        }

        if config.preset > 13 {
            return Err(Error::InvalidInput("Preset must be 0-13".to_string()));
        }

        if config.qp > 63 {
            return Err(Error::InvalidInput("QP must be 0-63".to_string()));
        }

        if config.bit_depth != 8 && config.bit_depth != 10 {
            return Err(Error::InvalidInput("Bit depth must be 8 or 10".to_string()));
        }

        unsafe {
            // Allocate encoder handle
            let mut handle: *mut EbComponentType = ptr::null_mut();
            let mut enc_config = std::mem::zeroed::<EbSvtAv1EncConfiguration>();

            // Initialize handle (this populates enc_config with defaults)
            let err = svt_av1_enc_init_handle(&mut handle, &mut enc_config);
            if err != 0 {
                return Err(Error::Codec(format!("Failed to initialize encoder handle: {}", err)));
            }

            // Configure encoder
            enc_config.enc_mode = config.preset as i8;
            enc_config.source_width = config.width;
            enc_config.source_height = config.height;
            enc_config.frame_rate_numerator = config.fps_num;
            enc_config.frame_rate_denominator = config.fps_den;
            enc_config.encoder_bit_depth = config.bit_depth;
            enc_config.encoder_color_format = 1; // YUV420 (EB_YUV420 = 1)
            enc_config.qp = config.qp;
            enc_config.rate_control_mode = 0; // CRF mode
            enc_config.tile_columns = config.tile_cols;
            enc_config.tile_rows = config.tile_rows;

            // Set configuration
            let err = svt_av1_enc_set_parameter(handle, &mut enc_config);
            if err != 0 {
                svt_av1_enc_deinit_handle(handle);
                return Err(Error::Codec(format!("Failed to set encoder parameters: {}", err)));
            }

            // Initialize encoder
            let err = svt_av1_enc_init(handle);
            if err != 0 {
                svt_av1_enc_deinit_handle(handle);
                return Err(Error::Codec(format!("Failed to initialize encoder: {}", err)));
            }

            Ok(Self {
                handle,
                width: config.width,
                height: config.height,
                frame_count: 0,
            })
        }
    }

    /// Convert error code to string
    fn error_string(code: i32) -> &'static str {
        match code as u32 {
            0 => "Success",
            0x80001000 => "Insufficient resources",
            0x80001001 => "Undefined error",
            0x80001004 => "Invalid component",
            0x80001005 => "Bad parameter",
            0x80002012 => "Destroy thread failed",
            0x80002021 => "Semaphore unresponsive",
            0x80002022 => "Destroy semaphore failed",
            0x80002030 => "Create mutex failed",
            0x80002031 => "Mutex unresponsive",
            0x80002032 => "Destroy mutex failed",
            _ => "Unknown error",
        }
    }
}

impl VideoEncoder for SvtAv1Encoder {
    fn send_frame(&mut self, frame: Option<ArcFrame>) -> Result<()> {
        unsafe {
            if let Some(frame) = frame {
                // Validate frame format
                if frame.format() != PixelFormat::Yuv420p {
                    return Err(Error::UnsupportedFormat("SVT-AV1 currently only supports YUV420p".to_string()));
                }

                if frame.width() != self.width
                    || frame.height() != self.height
                {
                    return Err(Error::InvalidInput(format!(
                        "Frame size {}x{} doesn't match encoder config {}x{}",
                        frame.width(),
                        frame.height(),
                        self.width,
                        self.height
                    )));
                }

                // Allocate input buffer
                let mut input_buffer = std::mem::zeroed::<EbBufferHeaderType>();
                let mut input_picture = std::mem::zeroed::<EbSvtIOFormat>();

                // Set up picture buffer
                // YUV420: Y plane + U plane + V plane
                let planes = frame.planes();
                if planes.len() != 3 {
                    return Err(Error::InvalidInput(format!("YUV420p requires 3 planes, got {}", planes.len())));
                }

                input_picture.y_stride = planes[0].stride() as u32;
                input_picture.cb_stride = planes[1].stride() as u32;
                input_picture.cr_stride = planes[2].stride() as u32;
                input_picture.luma = planes[0].data().as_ptr() as *mut u8;
                input_picture.cb = planes[1].data().as_ptr() as *mut u8;
                input_picture.cr = planes[2].data().as_ptr() as *mut u8;

                // Set buffer header
                input_buffer.size = std::mem::size_of::<EbBufferHeaderType>() as u32;
                input_buffer.p_buffer = &mut input_picture as *mut _ as *mut u8;
                input_buffer.n_filled_len = 0; // Not used for input
                input_buffer.n_alloc_len = 0;
                input_buffer.p_app_private = ptr::null_mut();
                input_buffer.wrapper_ptr = ptr::null_mut();
                input_buffer.pic_type = EbAv1PictureType_EB_AV1_INVALID_PICTURE;
                input_buffer.pts = self.frame_count as i64;

                // Send picture
                let err = svt_av1_enc_send_picture(self.handle, &mut input_buffer);
                if err != 0 {
                    return Err(Error::Codec(format!(
                        "Failed to send frame: {} ({})",
                        Self::error_string(err),
                        err
                    )));
                }

                self.frame_count += 1;
            } else {
                // Send EOS (end of stream)
                let mut eos_buffer = std::mem::zeroed::<EbBufferHeaderType>();
                eos_buffer.size = std::mem::size_of::<EbBufferHeaderType>() as u32;
                eos_buffer.n_alloc_len = 0;
                eos_buffer.n_filled_len = 0;
                eos_buffer.p_buffer = ptr::null_mut();
                eos_buffer.n_tick_count = 0;
                eos_buffer.p_app_private = ptr::null_mut();
                eos_buffer.wrapper_ptr = ptr::null_mut();
                eos_buffer.flags = 1; // EB_BUFFERFLAG_EOS

                let err = svt_av1_enc_send_picture(self.handle, &mut eos_buffer);
                if err != 0 {
                    return Err(Error::Codec(format!(
                        "Failed to send EOS: {} ({})",
                        Self::error_string(err),
                        err
                    )));
                }
            }

            Ok(())
        }
    }

    fn receive_packet(&mut self) -> Result<Option<Vec<u8>>> {
        unsafe {
            let mut output_buffer: *mut EbBufferHeaderType = ptr::null_mut();

            // Get packet
            let err = svt_av1_enc_get_packet(self.handle, &mut output_buffer, 0);

            if err != 0 {
                // Non-zero doesn't always mean error - might just mean no packet ready
                // EB_ErrorUndefined (0x80001001) means no packet available
                return Ok(None);
            }

            if output_buffer.is_null() {
                return Ok(None);
            }

            let buffer = &*output_buffer;

            // Check for EOS flag
            if buffer.flags & 1 != 0 {
                // EOS - release buffer and return None
                svt_av1_enc_release_out_buffer(&mut output_buffer);
                return Ok(None);
            }

            // Copy packet data
            let data = std::slice::from_raw_parts(
                buffer.p_buffer,
                buffer.n_filled_len as usize,
            );
            let packet = data.to_vec();

            // Release buffer
            svt_av1_enc_release_out_buffer(&mut output_buffer);

            Ok(Some(packet))
        }
    }
}

impl Drop for SvtAv1Encoder {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                // Deinit encoder
                let _ = svt_av1_enc_deinit(self.handle);

                // Deinit handle
                let _ = svt_av1_enc_deinit_handle(self.handle);

                self.handle = ptr::null_mut();
            }
        }
    }
}

// Safety: SVT-AV1 encoder is thread-safe per instance
unsafe impl Send for SvtAv1Encoder {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        // Invalid dimensions
        let config = SvtAv1Config {
            width: 0,
            height: 480,
            ..Default::default()
        };
        assert!(SvtAv1Encoder::new(config).is_err());

        // Invalid preset
        let config = SvtAv1Config {
            width: 640,
            height: 480,
            preset: 20,
            ..Default::default()
        };
        assert!(SvtAv1Encoder::new(config).is_err());

        // Invalid QP
        let config = SvtAv1Config {
            width: 640,
            height: 480,
            qp: 100,
            ..Default::default()
        };
        assert!(SvtAv1Encoder::new(config).is_err());
    }

    #[test]
    fn test_encoder_creation() {
        let config = SvtAv1Config {
            width: 640,
            height: 480,
            preset: 12, // Fastest for test
            ..Default::default()
        };

        let encoder = SvtAv1Encoder::new(config);
        assert!(encoder.is_ok());
    }
}
