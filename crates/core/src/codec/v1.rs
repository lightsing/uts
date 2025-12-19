//! Components for the version 1 OpenTimestamps serialization format.

mod attestation;
mod digest;
pub mod opcode;
mod timestamp;
mod detached_timestamp;

pub use attestation::{Attestation, AttestationTag};
pub use digest::DigestHeader;
pub use timestamp::Timestamp;
pub use detached_timestamp::DetachedTimestamp;
