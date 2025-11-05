//! mead-core: Memory-safe media processing library
//!
//! This library provides safe abstractions for media container parsing,
//! codec operations, and media processing pipelines.

#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    missing_debug_implementations
)]

pub mod container;
pub mod codec;
pub mod error;

pub use error::{Error, Result};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
