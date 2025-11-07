//! Video encoder implementations
//!
//! This module provides safe wrappers around encoder backends:
//! - SVT-AV1: Fast production encoder (default)
//! - rav1e: Pure Rust encoder (via mead-core)

pub mod svtav1;

use anyhow::Result;
use mead_core::ArcFrame;

/// Video encoder trait for unified interface
pub trait VideoEncoder {
    /// Send a frame for encoding (None signals end of stream)
    fn send_frame(&mut self, frame: Option<ArcFrame>) -> Result<()>;

    /// Receive an encoded packet (returns None when no more packets available)
    fn receive_packet(&mut self) -> Result<Option<Vec<u8>>>;

    /// Finish encoding and flush remaining packets
    fn finish(&mut self) -> Result<Vec<Vec<u8>>> {
        // Send EOS
        self.send_frame(None)?;

        // Collect remaining packets
        let mut packets = Vec::new();
        while let Some(packet) = self.receive_packet()? {
            packets.push(packet);
        }
        Ok(packets)
    }
}

/// Encoder selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderBackend {
    /// SVT-AV1 (fast, production-grade)
    SvtAv1,
    /// rav1e (pure Rust, memory-safe)
    Rav1e,
}

impl EncoderBackend {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "svt-av1" | "svtav1" => Some(Self::SvtAv1),
            "rav1e" => Some(Self::Rav1e),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SvtAv1 => "svt-av1",
            Self::Rav1e => "rav1e",
        }
    }
}
