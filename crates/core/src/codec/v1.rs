//! Components for the version 1 OpenTimestamps serialization format.

mod attestation;
mod detached_timestamp;
mod digest;
pub mod opcode;
mod timestamp;

pub use attestation::{
    Attestation, AttestationTag, BitcoinAttestation, EASAttestation, EASTimestamped,
    PendingAttestation, RawAttestation,
};
pub use detached_timestamp::DetachedTimestamp;
pub use digest::DigestHeader;
pub use timestamp::{Step, Timestamp, builder::TimestampBuilder};

/// Error indicating that finalization of a timestamp failed due to conflicting inputs.
#[derive(Debug)]
pub struct FinalizationError;

impl core::fmt::Display for FinalizationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "failed to finalize timestamp due to conflicting inputs")
    }
}

impl std::error::Error for FinalizationError {}

/// Trait for objects that may have input data.
pub trait MayHaveInput {
    /// Returns the input data for this object, if finalized.
    fn input(&self) -> Option<&[u8]>;
}

trait ToInput {
    fn to_input(&self) -> Option<&[u8]>;
}
impl<T: MayHaveInput> ToInput for T {
    fn to_input(&self) -> Option<&[u8]> {
        self.input()
    }
}
impl ToInput for [u8] {
    fn to_input(&self) -> Option<&[u8]> {
        Some(self)
    }
}
impl ToInput for Vec<u8> {
    fn to_input(&self) -> Option<&[u8]> {
        Some(self)
    }
}

/// Trait for objects that can be checked for consistency with another object.
#[allow(private_bounds)]
pub trait ConsistentWith<T: ToInput + ?Sized>: MayHaveInput {
    /// Checks if self is consistent with the given input.
    ///
    /// Note: Returns true if any of the inputs is not set.
    fn is_consistent_with(&self, other: &T) -> bool {
        self.input()
            .zip(other.to_input())
            .is_none_or(|(a, b)| a == b)
    }

    /// Checks if self is consistent with the given input.
    ///
    /// Note: Returns false if xor of the inputs is not set.
    fn is_consistent_with_strict(&self, other: &T) -> bool {
        self.input() == other.to_input()
    }
}

impl<T: MayHaveInput, U: ToInput + ?Sized> ConsistentWith<U> for T {}
