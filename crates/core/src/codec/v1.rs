//! Components for the version 1 OpenTimestamps serialization format.

mod attestation;
mod digest;
pub mod opcode;
mod timestamp;

pub use attestation::{Attestation, AttestationTag};
pub use digest::DigestHeader;
pub use timestamp::Timestamp;
