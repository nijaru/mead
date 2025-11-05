//! Codec implementations

pub mod av1;

use crate::Result;

/// Trait for video decoders
pub trait VideoDecoder {
    /// Decode a packet into a frame
    fn decode(&mut self, data: &[u8]) -> Result<Option<Frame>>;
}

/// Trait for video encoders
pub trait VideoEncoder {
    /// Encode a frame into packet data
    fn encode(&mut self, frame: &Frame) -> Result<Vec<u8>>;
}

/// A decoded video frame
#[derive(Debug, Clone)]
pub struct Frame {
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Pixel data (planar YUV for now)
    pub data: Vec<u8>,
    /// Presentation timestamp
    pub pts: Option<i64>,
}
