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
    /// Number of tiles horizontally (power of 2, 0 = auto)
    pub tile_cols: usize,
    /// Number of tiles vertically (power of 2, 0 = auto)
    pub tile_rows: usize,
    /// Number of threads (0 = auto-detect from CPU cores)
    pub threads: usize,
}

impl Default for Av1Config {
    fn default() -> Self {
        Self {
            speed: 6,
            quantizer: 100,
            bitrate_kbps: None,
            tile_cols: 0,  // Auto-calculate based on resolution
            tile_rows: 0,  // Auto-calculate based on resolution
            threads: 0,    // Auto-detect CPU cores
        }
    }
}

impl Av1Config {
    /// Calculate optimal tile configuration for given resolution
    ///
    /// Rules:
    /// - Tiles must be powers of 2
    /// - Each tile should be at least 256x256 pixels
    /// - Target ~4 tiles per CPU core for good parallelism
    pub fn calculate_tiles(width: u32, height: u32, threads: usize) -> (usize, usize) {
        // Don't create tiles smaller than 256x256
        let max_tile_cols = ((width / 256) as usize).next_power_of_two().max(1);
        let max_tile_rows = ((height / 256) as usize).next_power_of_two().max(1);

        // Target number of tiles based on threads (aim for 2-4 tiles per thread)
        let target_tiles = (threads * 2).max(4);

        // Find best split that fits constraints
        let mut best_cols = 1usize;
        let mut best_rows = 1usize;
        let mut best_diff = usize::MAX;

        for cols in [1usize, 2, 4, 8] {
            for rows in [1usize, 2, 4, 8] {
                if cols > max_tile_cols || rows > max_tile_rows {
                    continue;
                }

                let total = cols * rows;
                let diff = if total > target_tiles {
                    total - target_tiles
                } else {
                    target_tiles - total
                };

                if diff < best_diff {
                    best_diff = diff;
                    best_cols = cols;
                    best_rows = rows;
                }
            }
        }

        (best_cols, best_rows)
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
        // Auto-detect threads if not specified
        let threads = if config.threads == 0 {
            num_cpus::get()
        } else {
            config.threads
        };

        // Auto-calculate tiles if not specified
        let (tile_cols, tile_rows) = if config.tile_cols == 0 || config.tile_rows == 0 {
            Av1Config::calculate_tiles(width, height, threads)
        } else {
            (config.tile_cols, config.tile_rows)
        };

        tracing::debug!(
            "AV1 encoder config: {}x{}, speed={}, tiles={}x{}, threads={}",
            width, height, config.speed, tile_cols, tile_rows, threads
        );

        let mut enc_config = EncoderConfig {
            width: width as usize,
            height: height as usize,
            speed_settings: SpeedSettings::from_preset(config.speed),
            quantizer: config.quantizer as usize,
            tile_cols,
            tile_rows,
            ..Default::default()
        };

        if let Some(br) = config.bitrate_kbps {
            enc_config.bitrate = (br as i32) * 1000;
        }

        let cfg = Config::new()
            .with_encoder_config(enc_config)
            .with_threads(threads);

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
            tile_cols: 1,
            tile_rows: 1,
            threads: 2,
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

    #[test]
    fn test_tile_calculation() {
        // Small resolution: 640x480 with 4 threads
        // Limited by 256x256 minimum tile size (2.5 cols × 1.8 rows max)
        let (cols, rows) = Av1Config::calculate_tiles(640, 480, 4);
        assert!(cols * rows >= 1, "Should have at least 1 tile");
        assert!(cols * rows <= 8, "Shouldn't over-tile");
        println!("640x480@4t: {}x{} = {} tiles", cols, rows, cols * rows);

        // 1080p: 1920x1080 with 8 threads
        // Can support more tiles (7.5 cols × 4.2 rows max)
        let (cols, rows) = Av1Config::calculate_tiles(1920, 1080, 8);
        assert!(cols * rows >= 4, "Should have meaningful parallelism");
        assert!(cols * rows <= 32, "Shouldn't over-tile");
        println!("1920x1080@8t: {}x{} = {} tiles", cols, rows, cols * rows);

        // 4K: 3840x2160 with 16 threads
        // Good tile support (15 cols × 8.4 rows max)
        let (cols, rows) = Av1Config::calculate_tiles(3840, 2160, 16);
        assert!(cols * rows >= 8, "Should have good parallelism for 4K");
        assert!(cols * rows <= 64, "Shouldn't over-tile");
        println!("3840x2160@16t: {}x{} = {} tiles", cols, rows, cols * rows);
    }
}
