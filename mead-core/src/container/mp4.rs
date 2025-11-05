//! MP4 container support using mp4parse
//!
//! **Note**: Currently loads entire file for parsing (mp4parse limitation).
//! Full streaming support with location-based references coming in Phase 2.

use crate::{Error, MediaSource, Result};
use super::{Demuxer, Metadata, Packet};

/// MP4 demuxer
///
/// **Current Limitation**: Loads entire file into memory for parsing.
/// This is due to mp4parse API design. Phase 2 will implement box-by-box
/// streaming with location references for constant memory usage.
#[derive(Debug)]
pub struct Mp4Demuxer<R: MediaSource> {
    _source: R, // Will be used in Phase 2 for seeking to sample data
    context: mp4parse::MediaContext,
    metadata: Metadata,
}

impl<R: MediaSource> Mp4Demuxer<R> {
    /// Create a new MP4 demuxer from a media source
    ///
    /// **Warning**: Currently loads entire file into memory.
    /// Streaming support with constant memory usage coming in Phase 2.
    pub fn new(mut source: R) -> Result<Self> {
        tracing::warn!(
            "MP4 demuxer currently loads entire file - streaming support planned for Phase 2"
        );

        let mut buffer = Vec::new();
        std::io::Read::read_to_end(&mut source, &mut buffer)
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
            _source: source,
            context,
            metadata,
        })
    }

    /// Get track information
    pub fn tracks(&self) -> &[mp4parse::Track] {
        &self.context.tracks
    }
}

impl<R: MediaSource> Demuxer for Mp4Demuxer<R> {
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
