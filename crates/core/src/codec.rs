use crate::error::{DecodeError, EncodeError};
use alloc::alloc::Global;
use auto_impl::auto_impl;
use core::{alloc::Allocator, ops::RangeBounds};

mod proof;
pub use proof::{Proof, Version, VersionedProof};

mod imp;
#[cfg(feature = "std")]
pub use imp::{Reader, Writer};

/// Types and helpers for the version 1 serialization format.
pub mod v1;

/// Magic bytes that every proof must start with.
pub const MAGIC: &[u8; 31] = b"\x00OpenTimestamps\x00\x00Proof\x00\xbf\x89\xe2\xe8\x84\xe8\x92\x94";

/// Helper trait for writing OpenTimestamps primitives to a byte stream.
pub trait Encoder: Sized {
    /// Encodes a single byte to the writer.
    fn encode_byte(&mut self, byte: u8) -> Result<(), EncodeError>;

    /// Encodes a byte slice prefixed with its length.
    fn encode_bytes(&mut self, bytes: impl AsRef<[u8]>) -> Result<(), EncodeError> {
        let bytes = bytes.as_ref();
        self.encode(bytes.len())?;
        self.write_all(bytes)?;
        Ok(())
    }

    /// Writes the OpenTimestamps magic sequence to the stream.
    fn encode_magic(&mut self) -> Result<(), EncodeError> {
        self.write_all(&MAGIC[..])
    }

    /// Encodes a value implementing the [`Encode`] trait.
    fn encode(&mut self, value: impl Encode) -> Result<(), EncodeError> {
        value.encode(self)
    }

    // --- no_std feature compatibility ---
    fn write_all(&mut self, buf: impl AsRef<[u8]>) -> Result<(), EncodeError>;
}

/// Helper trait for reading OpenTimestamps primitives from a byte stream.
pub trait Decoder: Sized {
    /// Decodes a single byte from the reader.
    fn decode_byte(&mut self) -> Result<u8, DecodeError>;

    /// Decodes a value and ensures it falls within the supplied range.
    fn decode_ranged<T: Decode + PartialOrd>(
        &mut self,
        range: impl RangeBounds<T>,
    ) -> Result<T, DecodeError> {
        let val: T = self.decode()?;
        if range.contains(&val) {
            Ok(val)
        } else {
            Err(DecodeError::OutOfRange)
        }
    }

    /// Verifies that the next bytes in the stream match the magic sequence.
    fn assert_magic(&mut self) -> Result<(), DecodeError> {
        let mut buf = [0u8; MAGIC.len()];
        self.read_exact(&mut buf)?;
        if buf == *MAGIC {
            Ok(())
        } else {
            Err(DecodeError::BadMagic)
        }
    }

    /// Decodes a value implementing the [`Decode`] trait.
    fn decode<T: Decode>(&mut self) -> Result<T, DecodeError> {
        T::decode(self)
    }

    /// Decodes a trailing optional value implementing the [`Decode`] trait.
    ///
    /// See [`Decode::decode_trailing`] for details and caveats.
    fn decode_trailing<T: Decode>(&mut self) -> Result<Option<T>, DecodeError> {
        T::decode_trailing(self)
    }

    /// Decodes a value implementing the [`Decode`] trait.
    fn decode_in<T: DecodeIn<A>, A: Allocator>(&mut self, alloc: A) -> Result<T, DecodeError> {
        T::decode_in(self, alloc)
    }

    // --- no_std feature compatibility ---
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DecodeError>;
}

/// Serializes a value into an OpenTimestamps-compatible byte stream.
#[auto_impl(&, &mut, Box, Rc, Arc)]
pub trait Encode {
    fn encode(&self, writer: &mut impl Encoder) -> Result<(), EncodeError>;
}

/// Deserializes a value from an OpenTimestamps-compatible byte stream.
pub trait Decode: Sized {
    fn decode(decoder: &mut impl Decoder) -> Result<Self, DecodeError>;

    /// Decodes a trailing optional value implementing the [`Decode`] trait.
    ///
    /// This treats any `UnexpectedEof` error as an indication that the value is absent, returning `Ok(None)`.
    ///
    /// If the implementor returns `UnexpectedEof` for any reason other than the absence of the value,
    /// it should also override this method to avoid masking the error as `Ok(None)`.
    fn decode_trailing(decoder: &mut impl Decoder) -> Result<Option<Self>, DecodeError> {
        match Self::decode(decoder) {
            Ok(value) => Ok(Some(value)),
            Err(DecodeError::UnexpectedEof) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// Deserializes a value from an OpenTimestamps-compatible byte stream.
pub trait DecodeIn<A: Allocator>: Sized {
    /// See [`Decode::decode`] for details.
    fn decode_in(decoder: &mut impl Decoder, alloc: A) -> Result<Self, DecodeError>;

    /// See [`Decode::decode_trailing`] for details and caveats.
    fn decode_trailing(decoder: &mut impl Decoder, alloc: A) -> Result<Option<Self>, DecodeError> {
        match Self::decode_in(decoder, alloc) {
            Ok(value) => Ok(Some(value)),
            Err(DecodeError::UnexpectedEof) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl<T: DecodeIn<Global>> Decode for T {
    fn decode(decoder: &mut impl Decoder) -> Result<Self, DecodeError> {
        T::decode_in(decoder, Global)
    }

    fn decode_trailing(decoder: &mut impl Decoder) -> Result<Option<Self>, DecodeError> {
        match Self::decode_in(decoder, Global) {
            Ok(value) => Ok(Some(value)),
            Err(DecodeError::UnexpectedEof) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
