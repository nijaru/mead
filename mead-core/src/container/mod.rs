//! Container format handlers (MP4, WebM, MKV)

pub mod mp4;

use crate::Result;

/// Trait for container demuxers
pub trait Demuxer {
    /// Read the next packet from the container
    fn read_packet(&mut self) -> Result<Option<Packet>>;

    /// Get container metadata
    fn metadata(&self) -> &Metadata;
}

/// A packet of encoded media data
#[derive(Debug, Clone)]
pub struct Packet {
    /// Stream index this packet belongs to
    pub stream_index: usize,
    /// Packet data
    pub data: Vec<u8>,
    /// Presentation timestamp
    pub pts: Option<i64>,
    /// Decode timestamp
    pub dts: Option<i64>,
    /// Whether this is a keyframe
    pub is_keyframe: bool,
}

/// Container metadata
#[derive(Debug, Clone)]
pub struct Metadata {
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Number of streams
    pub stream_count: usize,
    /// Container format
    pub format: String,
}
