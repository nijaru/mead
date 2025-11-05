//! AV1 codec support using rav1e

use crate::{ArcFrame, Error, PixelFormat, Result};
use super::VideoEncoder;
use rav1e::prelude::*;

/// AV1 encoder configuration
#[derive(Debug, Clone)]
pub struct Av1Config {
    /// Encoding speed preset (0-10, lower is slower but better quality)
    pub speed: u8,
    /// Quantizer (0-255, lower is better quality)
    pub quantizer: u8,
    /// Bitrate in kilobits per second
    pub bitrate_kbps: Option<u32>,
}

impl Default for Av1Config {
    fn default() -> Self {
        Self {
            speed: 6,
            quantizer: 100,
            bitrate_kbps: None,
        }
    }
}

/// AV1 encoder
pub struct Av1Encoder {
    context: Context<u8>,
    width: u32,
    height: u32,
}

impl std::fmt::Debug for Av1Encoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Av1Encoder")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl Av1Encoder {
    /// Create a new AV1 encoder with default configuration
    pub fn new(width: u32, height: u32) -> Result<Self> {
        Self::with_config(width, height, Av1Config::default())
    }

    /// Create a new AV1 encoder with custom configuration
    pub fn with_config(width: u32, height: u32, config: Av1Config) -> Result<Self> {
        let mut enc_config = EncoderConfig {
            width: width as usize,
            height: height as usize,
            speed_settings: SpeedSettings::from_preset(config.speed),
            quantizer: config.quantizer as usize,
            ..Default::default()
        };

        if let Some(br) = config.bitrate_kbps {
            enc_config.bitrate = (br as i32) * 1000;
        }

        let cfg = Config::new().with_encoder_config(enc_config);

        let context = cfg
            .new_context()
            .map_err(|e| Error::Codec(format!("Failed to create AV1 encoder: {:?}", e)))?;

        Ok(Self {
            context,
            width,
            height,
        })
    }

    /// Flush the encoder and retrieve any remaining packets
    pub fn flush(&mut self) -> Result<Vec<Vec<u8>>> {
        self.context.flush();

        let mut packets = Vec::new();

        loop {
            match self.context.receive_packet() {
                Ok(packet) => {
                    packets.push(packet.data.to_vec());
                }
                Err(EncoderStatus::Encoded) => continue,
                Err(EncoderStatus::LimitReached) => break,
                Err(e) => {
                    return Err(Error::Codec(format!("Encoder error during flush: {:?}", e)))
                }
            }
        }

        Ok(packets)
    }
}

impl VideoEncoder for Av1Encoder {
    fn send_frame(&mut self, frame: Option<ArcFrame>) -> Result<()> {
        match frame {
            Some(arc_frame) => {
                // Validate dimensions
                if arc_frame.width() != self.width || arc_frame.height() != self.height {
                    return Err(Error::InvalidInput(format!(
                        "Frame dimensions {}x{} do not match encoder {}x{}",
                        arc_frame.width(),
                        arc_frame.height(),
                        self.width,
                        self.height
                    )));
                }

                // Validate format
                if arc_frame.format() != PixelFormat::Yuv420p {
                    return Err(Error::InvalidInput(
                        "AV1 encoder currently only supports YUV420p format".to_string()
                    ));
                }

                // Create rav1e frame
                let mut rav1e_frame = self.context.new_frame();

                // Copy Y plane
                let y_plane = arc_frame.plane_y().ok_or_else(|| {
                    Error::InvalidInput("Frame missing Y plane".to_string())
                })?;
                let rav1e_y = &mut rav1e_frame.planes[0];
                let y_stride = rav1e_y.cfg.stride;
                let y_data = rav1e_y.data_origin_mut();

                for y in 0..self.height as usize {
                    let row_offset = y * y_stride;
                    let row = y_plane.row(y);
                    y_data[row_offset..row_offset + row.len()].copy_from_slice(row);
                }

                // Copy U plane
                let u_plane = arc_frame.plane_u().ok_or_else(|| {
                    Error::InvalidInput("Frame missing U plane".to_string())
                })?;
                let rav1e_u = &mut rav1e_frame.planes[1];
                let u_stride = rav1e_u.cfg.stride;
                let u_data = rav1e_u.data_origin_mut();

                for y in 0..(self.height / 2) as usize {
                    let row_offset = y * u_stride;
                    let row = u_plane.row(y);
                    u_data[row_offset..row_offset + row.len()].copy_from_slice(row);
                }

                // Copy V plane
                let v_plane = arc_frame.plane_v().ok_or_else(|| {
                    Error::InvalidInput("Frame missing V plane".to_string())
                })?;
                let rav1e_v = &mut rav1e_frame.planes[2];
                let v_stride = rav1e_v.cfg.stride;
                let v_data = rav1e_v.data_origin_mut();

                for y in 0..(self.height / 2) as usize {
                    let row_offset = y * v_stride;
                    let row = v_plane.row(y);
                    v_data[row_offset..row_offset + row.len()].copy_from_slice(row);
                }

                // Send frame to encoder
                self.context
                    .send_frame(rav1e_frame)
                    .map_err(|e| Error::Codec(format!("Failed to send frame: {:?}", e)))?;

                Ok(())
            }
            None => {
                // Signal end-of-stream
                self.context.flush();
                Ok(())
            }
        }
    }

    fn receive_packet(&mut self) -> Result<Option<Vec<u8>>> {
        match self.context.receive_packet() {
            Ok(packet) => Ok(Some(packet.data.to_vec())),
            Err(EncoderStatus::Encoded) => {
                // Encoder is processing, try again
                self.receive_packet()
            }
            Err(EncoderStatus::NeedMoreData) => Ok(None),
            Err(EncoderStatus::LimitReached) => Ok(None),
            Err(e) => Err(Error::Codec(format!("Encoder error: {:?}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Frame;
    use std::sync::Arc;

    #[test]
    fn test_av1_encoder_creation() {
        let encoder = Av1Encoder::new(64, 64);
        assert!(encoder.is_ok());
    }

    #[test]
    fn test_av1_encoder_with_config() {
        let config = Av1Config {
            speed: 10,
            quantizer: 50,
            bitrate_kbps: Some(1000),
        };
        let encoder = Av1Encoder::with_config(64, 64, config);
        assert!(encoder.is_ok());
    }

    #[test]
    fn test_av1_send_receive() {
        let mut encoder = Av1Encoder::new(64, 64).unwrap();

        let frame = Arc::new(Frame::new(64, 64, PixelFormat::Yuv420p));

        // Send frame
        let result = encoder.send_frame(Some(frame));
        assert!(result.is_ok());

        // Try to receive (may or may not have packet yet)
        let _ = encoder.receive_packet();
    }

    #[test]
    fn test_av1_wrong_dimensions() {
        let mut encoder = Av1Encoder::new(64, 64).unwrap();

        let frame = Arc::new(Frame::new(32, 32, PixelFormat::Yuv420p));

        let result = encoder.send_frame(Some(frame));
        assert!(result.is_err());
    }

    #[test]
    fn test_av1_finish() {
        let mut encoder = Av1Encoder::new(64, 64).unwrap();

        // Send a few frames
        for _ in 0..3 {
            let frame = Arc::new(Frame::new(64, 64, PixelFormat::Yuv420p));
            encoder.send_frame(Some(frame)).unwrap();
        }

        // Finish encoding
        let packets = encoder.finish().unwrap();
        assert!(!packets.is_empty());
    }
}
