//! Low-level FFI bindings to SVT-AV1 encoder
//!
//! This crate provides unsafe bindings to the SVT-AV1 C library.
//! For safe wrappers, see the `mead` CLI crate.
//!
//! # Safety
//! This is a `-sys` crate containing raw FFI bindings. All functions are `unsafe`.
//! Users must ensure:
//! - Proper initialization and cleanup of encoder contexts
//! - Valid pointers and buffer sizes
//! - Thread safety when using encoder from multiple threads
//!
//! # Architecture Note
//! This crate is NOT used by `mead-core`, which remains `#![forbid(unsafe_code)]`.
//! Only the `mead` CLI uses these bindings for performance.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

// Include generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_exist() {
        // Verify key constants are generated
        let _ = SVT_AV1_ENC_ABI_VERSION;
        let _ = DEFAULT;
    }
}
