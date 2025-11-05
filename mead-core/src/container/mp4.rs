//! MP4 container support using mp4parse

use crate::{Error, Result};
use super::{Demuxer, Metadata, Packet};
use std::io::Read;

/// MP4 demuxer
#[derive(Debug)]
pub struct Mp4Demuxer {
    context: mp4parse::MediaContext,
    metadata: Metadata,
}

impl Mp4Demuxer {
    /// Create a new MP4 demuxer from a reader
    pub fn new<R: Read>(mut reader: R) -> Result<Self> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)
            .map_err(Error::Io)?;

        let mut cursor = std::io::Cursor::new(&buffer);
        let context = mp4parse::read_mp4(&mut cursor)
            .map_err(|e| Error::ContainerParse(format!("Failed to parse MP4: {:?}", e)))?;

        let duration_ms = context.tracks
            .first()
            .and_then(|track| track.edited_duration)
            .and_then(|duration| {
                context.timescale
                    .map(|ts| (duration.0 * 1000) / ts.0)
            });

        let stream_count = context.tracks.len();

        let metadata = Metadata {
            duration_ms,
            stream_count,
            format: "MP4".to_string(),
        };

        Ok(Self {
            context,
            metadata,
        })
    }

    /// Get track information
    pub fn tracks(&self) -> &[mp4parse::Track] {
        &self.context.tracks
    }
}

impl Demuxer for Mp4Demuxer {
    fn read_packet(&mut self) -> Result<Option<Packet>> {
        // mp4parse provides structure parsing but not sample data extraction
        // Full implementation requires tracking sample table offsets and reading from file
        // For Phase 1, metadata extraction is the priority
        Err(Error::UnsupportedFormat(
            "Packet reading not yet implemented - requires sample table parsing".to_string()
        ))
    }

    fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_mp4_demuxer_requires_valid_mp4() {
        let invalid_data = vec![0u8; 100];
        let cursor = Cursor::new(invalid_data);
        let result = Mp4Demuxer::new(cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_mp4_metadata_format() {
        let minimal_mp4 = create_minimal_mp4();
        let cursor = Cursor::new(minimal_mp4);

        if let Ok(demuxer) = Mp4Demuxer::new(cursor) {
            let metadata = demuxer.metadata();
            assert_eq!(metadata.format, "MP4");
        }
    }

    fn create_minimal_mp4() -> Vec<u8> {
        // Minimal valid MP4 structure with ftyp and moov boxes
        let mut data = Vec::new();

        // ftyp box (file type)
        data.extend_from_slice(&[
            0x00, 0x00, 0x00, 0x1c, // box size
            b'f', b't', b'y', b'p', // box type
            b'i', b's', b'o', b'm', // major brand
            0x00, 0x00, 0x02, 0x00, // minor version
            b'i', b's', b'o', b'm', // compatible brand
            b'i', b's', b'o', b'2', // compatible brand
            b'm', b'p', b'4', b'1', // compatible brand
        ]);

        // moov box (movie)
        data.extend_from_slice(&[
            0x00, 0x00, 0x00, 0x08, // box size (empty moov)
            b'm', b'o', b'o', b'v', // box type
        ]);

        data
    }
}
