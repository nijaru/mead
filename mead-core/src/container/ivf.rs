//! IVF (Indeo Video Format) container support for AV1/VP8/VP9
//!
//! IVF is a simple container format specifically designed for AV1, VP9, and VP8 video codecs.
//! It's widely supported by players (VLC, ffmpeg, etc.) and much simpler than MP4.
//!
//! Format:
//! - File header (32 bytes)
//! - Frame header (12 bytes) + frame data
//! - Repeat for each frame

use crate::{Error, Result};
use super::{Muxer, Packet};
use std::io::Write;

/// IVF file header (32 bytes)
#[derive(Debug)]
struct IvfHeader {
    signature: [u8; 4],      // "DKIF"
    version: u16,            // 0
    header_size: u16,        // 32
    fourcc: [u8; 4],         // "AV01" for AV1
    width: u16,
    height: u16,
    timebase_den: u32,       // Frame rate denominator
    timebase_num: u32,       // Frame rate numerator
    frame_count: u32,        // 0 initially, updated at end
    unused: u32,             // 0
}

impl IvfHeader {
    fn new(width: u16, height: u16, fps_num: u32, fps_den: u32) -> Self {
        Self {
            signature: *b"DKIF",
            version: 0,
            header_size: 32,
            fourcc: *b"AV01",  // AV1 codec
            width,
            height,
            timebase_den: fps_den,
            timebase_num: fps_num,
            frame_count: 0,
            unused: 0,
        }
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.signature)?;
        writer.write_all(&self.version.to_le_bytes())?;
        writer.write_all(&self.header_size.to_le_bytes())?;
        writer.write_all(&self.fourcc)?;
        writer.write_all(&self.width.to_le_bytes())?;
        writer.write_all(&self.height.to_le_bytes())?;
        writer.write_all(&self.timebase_den.to_le_bytes())?;
        writer.write_all(&self.timebase_num.to_le_bytes())?;
        writer.write_all(&self.frame_count.to_le_bytes())?;
        writer.write_all(&self.unused.to_le_bytes())?;
        Ok(())
    }
}

/// IVF frame header (12 bytes)
#[derive(Debug)]
struct IvfFrameHeader {
    frame_size: u32,
    timestamp: u64,
}

impl IvfFrameHeader {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.frame_size.to_le_bytes())?;
        writer.write_all(&self.timestamp.to_le_bytes())?;
        Ok(())
    }
}

/// IVF muxer for writing AV1 video
///
/// IVF is a simple container format designed for VP8, VP9, and AV1 codecs.
/// It's widely supported and much simpler than MP4.
///
/// # Example
/// ```no_run
/// use mead_core::container::ivf::IvfMuxer;
/// use mead_core::container::{Muxer, Packet};
/// use std::fs::File;
///
/// let file = File::create("output.ivf")?;
/// let mut muxer = IvfMuxer::new(file, 1920, 1080, 30, 1)?;
///
/// // Write encoded AV1 packets
/// let packet = Packet {
///     stream_index: 0,
///     data: vec![/* AV1 encoded data */],
///     pts: Some(0),
///     dts: None,
///     is_keyframe: true,
/// };
/// muxer.write_packet(packet)?;
///
/// muxer.finalize()?;
/// # Ok::<(), mead_core::Error>(())
/// ```
pub struct IvfMuxer<W: Write> {
    writer: W,
    header: IvfHeader,
    frame_count: u32,
    width: u16,
    height: u16,
}

impl<W: Write> IvfMuxer<W> {
    /// Create a new IVF muxer
    ///
    /// # Arguments
    /// * `writer` - Output writer
    /// * `width` - Video width in pixels
    /// * `height` - Video height in pixels
    /// * `fps_num` - Frame rate numerator (e.g., 30 for 30fps)
    /// * `fps_den` - Frame rate denominator (e.g., 1 for 30fps, 1001 for 29.97fps)
    pub fn new(mut writer: W, width: u16, height: u16, fps_num: u32, fps_den: u32) -> Result<Self> {
        tracing::info!(
            "Creating IVF muxer: {}x{} @ {}/{} fps",
            width, height, fps_num, fps_den
        );

        let header = IvfHeader::new(width, height, fps_num, fps_den);
        header.write(&mut writer)?;

        Ok(Self {
            writer,
            header,
            frame_count: 0,
            width,
            height,
        })
    }

    /// Get the current frame count
    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }

    /// Get video dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }
}

impl<W: Write> Muxer for IvfMuxer<W> {
    fn write_packet(&mut self, packet: Packet) -> Result<()> {
        // IVF only supports a single video stream
        if packet.stream_index != 0 {
            return Err(Error::InvalidInput(format!(
                "IVF only supports stream index 0, got {}",
                packet.stream_index
            )));
        }

        let timestamp = packet.pts.unwrap_or(self.frame_count as i64) as u64;

        let frame_header = IvfFrameHeader {
            frame_size: packet.data.len() as u32,
            timestamp,
        };

        frame_header.write(&mut self.writer)?;
        self.writer.write_all(&packet.data)?;

        self.frame_count += 1;

        if self.frame_count % 100 == 0 {
            tracing::debug!("Wrote frame {} to IVF", self.frame_count);
        }

        Ok(())
    }

    fn finalize(mut self) -> Result<()> {
        tracing::info!("Finalizing IVF file with {} frames", self.frame_count);

        // IVF doesn't require updating the header with frame count
        // (the frame_count field in the header is optional)
        // But we flush to ensure all data is written
        self.writer.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_ivf_header_size() {
        let header = IvfHeader::new(1920, 1080, 30, 1);
        let mut buf = Vec::new();
        header.write(&mut buf).unwrap();
        assert_eq!(buf.len(), 32, "IVF header should be exactly 32 bytes");
    }

    #[test]
    fn test_ivf_header_magic() {
        let header = IvfHeader::new(1920, 1080, 30, 1);
        let mut buf = Vec::new();
        header.write(&mut buf).unwrap();
        assert_eq!(&buf[0..4], b"DKIF", "IVF header should start with DKIF");
        assert_eq!(&buf[8..12], b"AV01", "IVF header should have AV01 fourcc");
    }

    #[test]
    fn test_ivf_muxer_creation() {
        let cursor = Cursor::new(Vec::new());
        let muxer = IvfMuxer::new(cursor, 1920, 1080, 30, 1).unwrap();
        assert_eq!(muxer.dimensions(), (1920, 1080));
        assert_eq!(muxer.frame_count(), 0);
    }

    #[test]
    fn test_ivf_write_packet() {
        let cursor = Cursor::new(Vec::new());
        let mut muxer = IvfMuxer::new(cursor, 1920, 1080, 30, 1).unwrap();

        let packet = Packet {
            stream_index: 0,
            data: vec![1, 2, 3, 4, 5],
            pts: Some(0),
            dts: None,
            is_keyframe: true,
        };

        muxer.write_packet(packet).unwrap();
        assert_eq!(muxer.frame_count(), 1);
    }

    #[test]
    fn test_ivf_multiple_packets() {
        let cursor = Cursor::new(Vec::new());
        let mut muxer = IvfMuxer::new(cursor, 640, 480, 24, 1).unwrap();

        for i in 0..10 {
            let packet = Packet {
                stream_index: 0,
                data: vec![i as u8; 100],
                pts: Some(i),
                dts: None,
                is_keyframe: i == 0,
            };
            muxer.write_packet(packet).unwrap();
        }

        assert_eq!(muxer.frame_count(), 10);
    }

    #[test]
    fn test_ivf_wrong_stream_index() {
        let cursor = Cursor::new(Vec::new());
        let mut muxer = IvfMuxer::new(cursor, 1920, 1080, 30, 1).unwrap();

        let packet = Packet {
            stream_index: 1,  // IVF only supports stream 0
            data: vec![1, 2, 3],
            pts: Some(0),
            dts: None,
            is_keyframe: true,
        };

        assert!(muxer.write_packet(packet).is_err());
    }
}
