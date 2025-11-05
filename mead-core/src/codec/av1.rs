//! AV1 codec support using rav1e

use crate::{Error, Result};
use super::{Frame, VideoEncoder};

/// AV1 encoder
pub struct Av1Encoder {
    // TODO: Implement using rav1e
    _phantom: std::marker::PhantomData<()>,
}

impl Av1Encoder {
    /// Create a new AV1 encoder
    pub fn new(_width: u32, _height: u32) -> Result<Self> {
        Err(Error::UnsupportedFormat("AV1 encoder not yet implemented".to_string()))
    }
}

impl VideoEncoder for Av1Encoder {
    fn encode(&mut self, _frame: &Frame) -> Result<Vec<u8>> {
        Err(Error::UnsupportedFormat("Not yet implemented".to_string()))
    }
}
