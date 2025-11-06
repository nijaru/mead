//! Opus audio codec support using audiopus

use crate::{Error, Result};
use super::AudioDecoder;
use audiopus::{coder::Decoder as OpusDecoder, SampleRate, Channels};

/// Opus audio decoder
pub struct OpusDecoderImpl {
    decoder: OpusDecoder,
}

impl std::fmt::Debug for OpusDecoderImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpusDecoder").finish()
    }
}

impl OpusDecoderImpl {
    /// Create a new Opus decoder
    pub fn new(sample_rate: SampleRate, channels: Channels) -> Result<Self> {
        let decoder = OpusDecoder::new(sample_rate, channels)
            .map_err(|e| Error::Codec(format!("Failed to create Opus decoder: {:?}", e)))?;

        Ok(Self { decoder })
    }
}

impl AudioDecoder for OpusDecoderImpl {
    fn decode(&mut self, data: &[u8]) -> Result<Option<Vec<f32>>> {
        // For simplicity, assume we have enough space for decoded samples
        // In a real implementation, you'd need to know the frame size
        let mut output = vec![0.0f32; 4096]; // Temporary buffer

        match self.decoder.decode_float(Some(data), &mut output, false) {
            Ok(samples_decoded) => {
                if samples_decoded > 0 {
                    output.truncate(samples_decoded);
                    Ok(Some(output))
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(Error::Codec(format!("Opus decoding error: {:?}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opus_decoder_creation() {
        let decoder = OpusDecoderImpl::new(SampleRate::Hz48000, Channels::Stereo);
        assert!(decoder.is_ok());
    }
}
