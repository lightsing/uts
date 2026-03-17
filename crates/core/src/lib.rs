//! # Universal Timestamps Core Library

extern crate core;

mod tracing;

#[cfg(test)]
pub mod fixtures;

/// Encoding and decoding support for OpenTimestamps proofs.
pub mod codec;
/// Error types raised by codec operations.
pub mod error;
pub mod utils;
#[cfg(feature = "verifier")]
pub mod verifier;

/// Re-export the allocator2 API for use.
#[cfg(not(feature = "nightly"))]
pub mod alloc {
    pub use allocator_api2::{alloc::*, *};
}

#[cfg(feature = "nightly")]
pub mod alloc {
    pub use alloc::*;
    pub use core::alloc::*;
}
