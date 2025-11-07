//! Y4M (YUV4MPEG2) container support for raw video
//!
//! Y4M is a simple uncompressed video format used by video processing tools.
//! It consists of a text header followed by raw YUV frames.
//!
//! Common workflow:
//! ```bash
//! ffmpeg -i input.mp4 -f yuv4mpeg - | mead encode -o output.ivf --codec av1
//! ```

use crate::{Error, Frame, PixelFormat, Result};
use std::io::Read;

/// Y4M demuxer for reading raw YUV video
///
/// Y4M is a simple format with text headers and raw YUV data.
/// Widely used in video processing pipelines.
///
/// # Example
/// ```no_run
/// use mead_core::container::y4m::Y4mDemuxer;
/// use std::fs::File;
///
/// let file = File::open("input.y4m")?;
/// let mut demuxer = Y4mDemuxer::new(file)?;
///
/// let (fps_num, fps_den) = demuxer.framerate();
/// println!("Video: {}x{} @ {}/{} fps",
///     demuxer.width(),
///     demuxer.height(),
///     fps_num,
///     fps_den
/// );
///
/// while let Some(frame) = demuxer.read_frame()? {
///     // Process frame
/// }
/// # Ok::<(), mead_core::Error>(())
/// ```
pub struct Y4mDemuxer<R: Read> {
    decoder: y4m::Decoder<R>,
    width: u32,
    height: u32,
    framerate: (u64, u64),
    pixel_format: PixelFormat,
    frame_count: u64,
}

impl<R: Read> Y4mDemuxer<R> {
    /// Create a new Y4M demuxer
    pub fn new(reader: R) -> Result<Self> {
        let decoder = y4m::Decoder::new(reader)
            .map_err(|e| Error::ContainerParse(format!("Failed to parse Y4M: {}", e)))?;

        let width = decoder.get_width();
        let height = decoder.get_height();
        let framerate = decoder.get_framerate();
        let colorspace = decoder.get_colorspace();

        tracing::info!(
            "Y4M: {}x{} @ {}/{} fps, colorspace: {:?}",
            width,
            height,
            framerate.num,
            framerate.den,
            colorspace
        );

        // Map Y4M colorspace to our PixelFormat
        let pixel_format = match colorspace {
            y4m::Colorspace::C420 | y4m::Colorspace::C420jpeg | y4m::Colorspace::C420paldv | y4m::Colorspace::C420mpeg2 => {
                PixelFormat::Yuv420p
            }
            y4m::Colorspace::C422 => PixelFormat::Yuv422p,
            y4m::Colorspace::C444 => PixelFormat::Yuv444p,
            other => {
                return Err(Error::InvalidInput(format!(
                    "Unsupported Y4M colorspace: {:?}",
                    other
                )));
            }
        };

        Ok(Self {
            decoder,
            width: width as u32,
            height: height as u32,
            framerate: (framerate.num as u64, framerate.den as u64),
            pixel_format,
            frame_count: 0,
        })
    }

    /// Get video width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get video height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get framerate as (numerator, denominator)
    pub fn framerate(&self) -> (u64, u64) {
        self.framerate
    }

    /// Get pixel format
    pub fn pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }

    /// Get number of frames read so far
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Read the next frame
    ///
    /// Returns `Ok(None)` when EOF is reached.
    pub fn read_frame(&mut self) -> Result<Option<Frame>> {
        let y4m_frame = match self.decoder.read_frame() {
            Ok(frame) => frame,
            Err(y4m::Error::EOF) => return Ok(None),
            Err(e) => return Err(Error::ContainerParse(format!("Y4M read error: {}", e))),
        };

        // Extract plane data
        let y_plane = y4m_frame.get_y_plane();
        let u_plane = y4m_frame.get_u_plane();
        let v_plane = y4m_frame.get_v_plane();

        let mut frame = Frame::new(self.width, self.height, self.pixel_format);

        // Copy Y plane
        frame.planes_mut()[0].data_mut().copy_from_slice(y_plane);

        // Copy U plane
        frame.planes_mut()[1].data_mut()[..u_plane.len()].copy_from_slice(u_plane);

        // Copy V plane
        frame.planes_mut()[2].data_mut()[..v_plane.len()].copy_from_slice(v_plane);

        self.frame_count += 1;

        if self.frame_count % 100 == 0 {
            tracing::debug!("Read {} frames from Y4M", self.frame_count);
        }

        Ok(Some(frame))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn create_minimal_y4m() -> Vec<u8> {
        // Minimal Y4M file: header + one 2x2 YUV420p frame
        let header = b"YUV4MPEG2 W2 H2 F25:1 Ip A0:0 C420jpeg\n";
        let frame_header = b"FRAME\n";

        // Y plane: 2x2 = 4 pixels
        let y_data = vec![128u8; 4];
        // U plane: 1x1 = 1 pixel (420 subsampling)
        let u_data = vec![128u8; 1];
        // V plane: 1x1 = 1 pixel
        let v_data = vec![128u8; 1];

        let mut data = Vec::new();
        data.extend_from_slice(header);
        data.extend_from_slice(frame_header);
        data.extend_from_slice(&y_data);
        data.extend_from_slice(&u_data);
        data.extend_from_slice(&v_data);

        data
    }

    #[test]
    fn test_y4m_demuxer_creation() {
        let data = create_minimal_y4m();
        let cursor = Cursor::new(data);
        let demuxer = Y4mDemuxer::new(cursor).unwrap();

        assert_eq!(demuxer.width(), 2);
        assert_eq!(demuxer.height(), 2);
        assert_eq!(demuxer.framerate(), (25, 1));
        assert_eq!(demuxer.pixel_format(), PixelFormat::Yuv420p);
    }

    #[test]
    fn test_y4m_read_frame() {
        let data = create_minimal_y4m();
        let cursor = Cursor::new(data);
        let mut demuxer = Y4mDemuxer::new(cursor).unwrap();

        let frame = demuxer.read_frame().unwrap();
        assert!(frame.is_some());

        let frame = frame.unwrap();
        assert_eq!(frame.width(), 2);
        assert_eq!(frame.height(), 2);
        assert_eq!(frame.format(), PixelFormat::Yuv420p);

        // EOF
        let frame2 = demuxer.read_frame().unwrap();
        assert!(frame2.is_none());
    }

    #[test]
    fn test_y4m_frame_count() {
        let data = create_minimal_y4m();
        let cursor = Cursor::new(data);
        let mut demuxer = Y4mDemuxer::new(cursor).unwrap();

        assert_eq!(demuxer.frame_count(), 0);
        demuxer.read_frame().unwrap();
        assert_eq!(demuxer.frame_count(), 1);
    }
}
