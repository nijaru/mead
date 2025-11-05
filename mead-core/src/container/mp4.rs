//! MP4 container support using mp4parse

use crate::{Error, Result};
use super::{Demuxer, Metadata, Packet};

/// MP4 demuxer
pub struct Mp4Demuxer {
    // TODO: Implement using mp4parse
    _phantom: std::marker::PhantomData<()>,
}

impl Mp4Demuxer {
    /// Create a new MP4 demuxer from a reader
    pub fn new<R: std::io::Read>(_reader: R) -> Result<Self> {
        // Placeholder implementation
        Err(Error::UnsupportedFormat("MP4 not yet implemented".to_string()))
    }
}

impl Demuxer for Mp4Demuxer {
    fn read_packet(&mut self) -> Result<Option<Packet>> {
        Err(Error::UnsupportedFormat("Not yet implemented".to_string()))
    }

    fn metadata(&self) -> &Metadata {
        unimplemented!("MP4 metadata not yet implemented")
    }
}
