//! AV1 codec support using rav1e

use crate::{Error, Result};
use super::{Frame, VideoEncoder};
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
    fn encode(&mut self, frame: &Frame) -> Result<Vec<u8>> {
        if frame.width != self.width || frame.height != self.height {
            return Err(Error::InvalidInput(format!(
                "Frame dimensions {}x{} do not match encoder {}x{}",
                frame.width, frame.height, self.width, self.height
            )));
        }

        let mut rav1e_frame = self.context.new_frame();

        let y_size = (self.width * self.height) as usize;

        if frame.data.len() < y_size {
            return Err(Error::InvalidInput(
                "Frame data too small for Y plane".to_string()
            ));
        }

        let plane = &mut rav1e_frame.planes[0];
        let stride = plane.cfg.stride;
        let plane_data = plane.data_origin_mut();

        for y in 0..self.height as usize {
            let row_offset = y * stride;
            let src_offset = y * self.width as usize;
            let row_len = self.width as usize;
            plane_data[row_offset..row_offset + row_len]
                .copy_from_slice(&frame.data[src_offset..src_offset + row_len]);
        }

        self.context
            .send_frame(rav1e_frame)
            .map_err(|e| Error::Codec(format!("Failed to send frame: {:?}", e)))?;

        match self.context.receive_packet() {
            Ok(packet) => Ok(packet.data.to_vec()),
            Err(EncoderStatus::Encoded) => {
                Ok(Vec::new())
            }
            Err(EncoderStatus::NeedMoreData) => {
                Ok(Vec::new())
            }
            Err(e) => Err(Error::Codec(format!("Encoder error: {:?}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_av1_encode_frame() {
        let mut encoder = Av1Encoder::new(64, 64).unwrap();

        let frame = Frame {
            width: 64,
            height: 64,
            data: vec![128; 64 * 64],
            pts: Some(0),
        };

        let result = encoder.encode(&frame);
        assert!(result.is_ok());
    }

    #[test]
    fn test_av1_encode_wrong_dimensions() {
        let mut encoder = Av1Encoder::new(64, 64).unwrap();

        let frame = Frame {
            width: 32,
            height: 32,
            data: vec![128; 32 * 32],
            pts: Some(0),
        };

        let result = encoder.encode(&frame);
        assert!(result.is_err());
    }
}
