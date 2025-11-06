//! Codec implementations

pub mod av1;
pub mod aac;
pub mod opus;

use crate::{ArcFrame, Result};

/// Trait for video decoders
pub trait VideoDecoder {
    /// Decode a packet into a frame
    fn decode(&mut self, data: &[u8]) -> Result<Option<ArcFrame>>;
}

/// Trait for audio decoders
pub trait AudioDecoder {
    /// Decode audio data into PCM samples (f32, interleaved)
    fn decode(&mut self, data: &[u8]) -> Result<Option<Vec<f32>>>;
}

/// Trait for video encoders (send-receive pattern)
pub trait VideoEncoder {
    /// Send a frame to the encoder (None signals end-of-stream)
    fn send_frame(&mut self, frame: Option<ArcFrame>) -> Result<()>;

    /// Receive an encoded packet (None means encoder needs more frames)
    fn receive_packet(&mut self) -> Result<Option<Vec<u8>>>;

    /// Convenience method to flush all remaining packets
    fn finish(&mut self) -> Result<Vec<Vec<u8>>> {
        self.send_frame(None)?;
        let mut packets = Vec::new();
        while let Some(packet) = self.receive_packet()? {
            packets.push(packet);
        }
        Ok(packets)
    }
}
