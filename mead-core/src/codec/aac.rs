//! AAC audio codec support using symphonia
//!
//! Note: This is a placeholder implementation. Proper AAC decoding
//! requires ADTS header parsing and packetization.

use crate::{Error, Result};
use super::AudioDecoder;

/// AAC audio decoder (placeholder)
pub struct AacDecoder;

impl std::fmt::Debug for AacDecoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AacDecoder").finish()
    }
}

impl AacDecoder {
    /// Create a new AAC decoder
    pub fn new() -> Result<Self> {
        // AAC decoding requires proper implementation with:
        // 1. ADTS header parsing
        // 2. Packetization
        // 3. Symphonia format probing
        Err(Error::Codec("AAC decoder not yet implemented - requires ADTS parsing".to_string()))
    }
}

impl AudioDecoder for AacDecoder {
    fn decode(&mut self, _data: &[u8]) -> Result<Option<Vec<f32>>> {
        Err(Error::Codec("AAC decoding requires proper packetization".to_string()))
    }
}
