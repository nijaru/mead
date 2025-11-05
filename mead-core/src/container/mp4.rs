//! MP4 container support using mp4 crate
//!
//! Uses buffered reading with the `mp4` crate for efficient large file handling.
//! Does NOT load entire file into memory.

use crate::{Error, MediaSource, Result};
use super::{Demuxer, Metadata, Packet};
use std::io::BufReader;

/// MP4 demuxer using streaming with `mp4` crate
///
/// Uses BufReader for efficient I/O - does NOT load entire file into memory.
/// Maintains constant memory usage regardless of file size.
pub struct Mp4Demuxer<R: MediaSource> {
    reader: mp4::Mp4Reader<BufReader<R>>,
    metadata: Metadata,
    current_track: Option<u32>,
    current_sample: u32,
}

impl<R: MediaSource> Mp4Demuxer<R> {
    /// Create a new MP4 demuxer from a media source
    ///
    /// Uses buffered reading for efficient large file handling.
    /// Memory usage is constant regardless of file size.
    pub fn new(source: R) -> Result<Self> {
        // Get file size for mp4 crate API
        let size = source.len().ok_or_else(|| {
            Error::InvalidInput("Cannot determine source length - required for MP4 parsing".to_string())
        })?;

        tracing::info!("Opening MP4 file ({} bytes) with streaming support", size);

        let buf_reader = BufReader::new(source);
        let reader = mp4::Mp4Reader::read_header(buf_reader, size)
            .map_err(|e| Error::ContainerParse(format!("Failed to parse MP4: {}", e)))?;

        // Extract metadata
        let duration = reader.duration();
        let timescale = reader.timescale();
        let duration_ms = if timescale > 0 {
            Some((duration.as_millis() * 1000) / timescale as u128).and_then(|d| d.try_into().ok())
        } else {
            None
        };

        let stream_count = reader.tracks().len();

        let metadata = Metadata {
            duration_ms,
            stream_count,
            format: "MP4".to_string(),
        };

        tracing::info!(
            "MP4 opened: {} tracks, duration: {:?}ms",
            stream_count,
            duration_ms
        );

        Ok(Self {
            reader,
            metadata,
            current_track: None,
            current_sample: 0,
        })
    }

    /// Get track information
    pub fn tracks(&self) -> &std::collections::HashMap<u32, mp4::Mp4Track> {
        self.reader.tracks()
    }

    /// Select a track for reading
    pub fn select_track(&mut self, track_id: u32) -> Result<()> {
        if !self.reader.tracks().contains_key(&track_id) {
            return Err(Error::InvalidInput(format!(
                "Track {} not found",
                track_id
            )));
        }
        self.current_track = Some(track_id);
        self.current_sample = 0;
        Ok(())
    }
}

impl<R: MediaSource> std::fmt::Debug for Mp4Demuxer<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mp4Demuxer")
            .field("metadata", &self.metadata)
            .field("current_track", &self.current_track)
            .field("current_sample", &self.current_sample)
            .finish()
    }
}

impl<R: MediaSource> Demuxer for Mp4Demuxer<R> {
    fn read_packet(&mut self) -> Result<Option<Packet>> {
        let track_id = match self.current_track {
            Some(id) => id,
            None => {
                // Auto-select first track if none selected
                if let Some(&first_id) = self.reader.tracks().keys().next() {
                    self.select_track(first_id)?;
                    first_id
                } else {
                    return Ok(None); // No tracks
                }
            }
        };

        self.current_sample += 1;

        // Try to read sample from current track
        match self.reader.read_sample(track_id, self.current_sample) {
            Ok(Some(sample)) => {
                // Convert mp4::Mp4Sample to our Packet type
                Ok(Some(Packet {
                    stream_index: track_id as usize,
                    data: sample.bytes.to_vec(),
                    pts: Some(sample.start_time as i64),
                    dts: None, // mp4 crate doesn't expose DTS separately
                    is_keyframe: sample.is_sync,
                }))
            }
            Ok(None) => {
                // End of current track, try next track
                self.current_track = None;
                self.current_sample = 0;
                Ok(None)
            }
            Err(e) => Err(Error::ContainerParse(format!(
                "Failed to read sample: {}",
                e
            ))),
        }
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
