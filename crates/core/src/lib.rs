#![feature(exact_bitshifts)]
#![feature(allocator_api)]
#![cfg_attr(not(feature = "std"), no_std)]
//! # Universal Timestamps Core Library

extern crate alloc;
extern crate core;

mod tracing;

#[cfg(test)]
pub mod fixtures;

/// Encoding and decoding support for OpenTimestamps proofs.
pub mod codec;
/// Error types raised by codec operations.
pub mod error;
pub mod utils;
