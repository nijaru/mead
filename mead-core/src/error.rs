//! Error types for mead

use thiserror::Error;

/// Result type alias using mead's Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during media processing
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error occurred
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Container parsing error
    #[error("Container parsing failed: {0}")]
    ContainerParse(String),

    /// Codec error
    #[error("Codec error: {0}")]
    Codec(String),

    /// Unsupported format
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
