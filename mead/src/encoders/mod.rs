//! Video encoder implementations
//!
//! This module provides safe wrappers around encoder backends:
//! - SVT-AV1: Fast production encoder (default)
//! - rav1e: Pure Rust encoder (via mead-core)

pub mod svtav1;

// Re-export the VideoEncoder trait from mead-core for unified interface
pub use mead_core::codec::VideoEncoder;

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
