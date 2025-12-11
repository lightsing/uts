use crate::{
    codec::{Codec, Decode, Decoder, Encode, Encoder},
    error::{DecodeError, EncodeError},
};
use std::fmt;

/// Version number of the serialization format.
pub type Version = u32;

/// Trait implemented by proof payloads for a specific serialization version.
pub trait Proof: Codec {
    /// Version identifier that must match the encoded proof.
    const VERSION: Version;
}

/// Wrapper that prefixes a proof with its version and magic bytes.
#[derive(Clone, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct VersionedProof<T: Proof>(pub T);

impl<T: Proof> Decode for VersionedProof<T> {
    fn decode(mut reader: impl Decoder) -> Result<Self, DecodeError> {
        reader.assert_magic()?;
        let version: Version = reader.decode()?;
        if version != T::VERSION {
            return Err(DecodeError::BadVersion);
        }
        Ok(VersionedProof(T::decode(&mut reader)?))
    }
}

impl<T: Proof> Encode for VersionedProof<T> {
    fn encode(&self, mut writer: impl Encoder) -> Result<(), EncodeError> {
        writer.encode_magic()?;
        writer.encode(T::VERSION)?;
        self.0.encode(writer)
    }
}

impl<T: Proof + fmt::Display> fmt::Display for VersionedProof<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Version {} Proof {}", T::VERSION, self.0)
    }
}
