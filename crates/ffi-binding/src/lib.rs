//! # UTS FFI Binding
//!
//! UniFFI-based foreign function interface for the [`uts-core`] library.

mod error;
pub use error::UtsError;

pub mod codec;
mod primitives;

uniffi::setup_scaffolding!("uts");
