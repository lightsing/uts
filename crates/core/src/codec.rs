use crate::error::{DecodeError, EncodeError};
use auto_impl::auto_impl;
use std::{
    io::{BufRead, Write},
    ops::RangeBounds,
};

mod proof;
pub use proof::{Proof, Version, VersionedProof};

mod primitives;

/// Types and helpers for the version 1 serialization format.
pub mod v1;

/// Magic bytes that every proof must start with.
pub const MAGIC: &[u8; 31] = b"\x00OpenTimestamps\x00\x00Proof\x00\xbf\x89\xe2\xe8\x84\xe8\x92\x94";

/// Helper trait for writing OpenTimestamps primitives to a byte stream.
pub trait Encoder: Write {
    /// Encodes a single byte to the writer.
    fn encode_byte(&mut self, byte: u8) -> Result<(), EncodeError> {
        self.write_all(&[byte])?;
        Ok(())
    }

    /// Encodes a byte slice prefixed with its length.
    fn encode_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        self.encode(bytes.len())?;
        self.write_all(bytes)?;
        Ok(())
    }

    /// Writes the OpenTimestamps magic sequence to the stream.
    fn encode_magic(&mut self) -> Result<(), EncodeError> {
        self.write_all(MAGIC)?;
        Ok(())
    }

    /// Encodes a value implementing the [`Encode`] trait.
    #[inline]
    fn encode(&mut self, value: impl Encode) -> Result<(), EncodeError> {
        value.encode(self)
    }
}

impl<W: Write> Encoder for W {}

/// Helper trait for reading OpenTimestamps primitives from a byte stream.
pub trait Decoder: BufRead {
    /// Decodes a single byte from the reader.
    fn decode_byte(&mut self) -> Result<u8, DecodeError> {
        let mut byte = [0];
        self.read_exact(&mut byte)?;
        Ok(byte[0])
    }

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
    #[inline]
    fn decode<T: Decode>(&mut self) -> Result<T, DecodeError> {
        T::decode(self)
    }
}

impl<R: BufRead> Decoder for R {}

/// Marker trait for types supporting both [`Encode`] and [`Decode`].
pub trait Codec: Encode + Decode {}

impl<T: Encode + Decode> Codec for T {}

/// Serializes a value into an OpenTimestamps-compatible byte stream.
#[auto_impl(&, &mut, Box, Rc, Arc)]
pub trait Encode {
    fn encode(&self, writer: impl Encoder) -> Result<(), EncodeError>;
}

/// Deserializes a value from an OpenTimestamps-compatible byte stream.
pub trait Decode: Sized {
    fn decode(reader: impl Decoder) -> Result<Self, DecodeError>;
}
