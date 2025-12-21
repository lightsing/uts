#![feature(exact_bitshifts)]
//! # Universal Timestamps Core Library

mod tracing;

#[cfg(test)]
pub mod fixtures;

/// Encoding and decoding support for OpenTimestamps proofs.
pub mod codec;
/// Error types raised by codec operations.
pub mod error;
pub mod utils;
