use crate::{
    alloc::{Allocator, Global},
    codec::{DecodeIn, Decoder, Encode, Encoder},
    error::{DecodeError, EncodeError},
};
use core::fmt;

/// Version number of the serialization format.
pub type Version = u32;

/// Trait implemented by proof payloads for a specific serialization version.
pub trait Proof<A: Allocator = Global>: Encode + DecodeIn<A> {
    /// Version identifier that must match the encoded proof.
    const VERSION: Version;
}

/// Wrapper that prefixes a proof with its version and magic bytes.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VersionedProof<T: Proof<A>, A: Allocator = Global> {
    pub proof: T,
    allocator: A,
}

impl<T: Proof<Global>> VersionedProof<T, Global> {
    /// Creates a new versioned proof with the global allocator.
    pub fn new(proof: T) -> Self {
        VersionedProof {
            proof,
            allocator: Global,
        }
    }
}

impl<T: Proof<A>, A: Allocator> VersionedProof<T, A> {
    /// Creates a new versioned proof with the specified allocator.
    pub fn new_with_allocator(proof: T, allocator: A) -> Self {
        VersionedProof { proof, allocator }
    }

    /// Returns a reference to the proof payload.
    pub fn proof(&self) -> &T {
        &self.proof
    }

    /// Returns a reference to the allocator used by this proof.
    pub fn allocator(&self) -> &A {
        &self.allocator
    }
}

impl<T: Proof<A>, A: Allocator + Clone> DecodeIn<A> for VersionedProof<T, A> {
    fn decode_in(decoder: &mut impl Decoder, alloc: A) -> Result<Self, DecodeError> {
        decoder.assert_magic()?;
        let version: Version = decoder.decode()?;
        if version != T::VERSION {
            return Err(DecodeError::BadVersion);
        }
        Ok(VersionedProof {
            proof: T::decode_in(decoder, alloc.clone())?,
            allocator: alloc,
        })
    }
}

impl<T: Proof<A>, A: Allocator> Encode for VersionedProof<T, A> {
    fn encode(&self, encoder: &mut impl Encoder) -> Result<(), EncodeError> {
        encoder.encode_magic()?;
        encoder.encode(T::VERSION)?;
        self.proof.encode(encoder)
    }
}

impl<T: Proof<A> + fmt::Display, A: Allocator> fmt::Display for VersionedProof<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Version {} Proof {}", T::VERSION, self.proof)
    }
}
