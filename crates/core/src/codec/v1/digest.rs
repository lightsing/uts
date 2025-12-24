use crate::{
    codec::{DecodeIn, Decoder, Encode, Encoder, v1::opcode::DigestOp},
    error::{DecodeError, EncodeError},
    utils::Hexed,
};
use alloc::alloc::Allocator;
use core::fmt;

/// Header describing the digest that anchors a timestamp.
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DigestHeader {
    kind: DigestOp,
    digest: [u8; 32],
}

impl fmt::Debug for DigestHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DigestHeader")
            .field("kind", &self.kind)
            .field("digest", &Hexed(self.digest()))
            .finish()
    }
}

impl fmt::Display for DigestHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.kind, Hexed(self.digest()))
    }
}

impl DigestHeader {
    /// Returns the digest opcode recorded in the header.
    pub fn kind(&self) -> DigestOp {
        self.kind
    }

    /// Returns the digest bytes trimmed to the opcode's output size.
    pub fn digest(&self) -> &[u8] {
        &self.digest[..self.kind.output_size()]
    }
}

impl Encode for DigestHeader {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(writer), err))]
    #[inline]
    fn encode(&self, encoder: &mut impl Encoder) -> Result<(), EncodeError> {
        encoder.encode(self.kind)?;
        encoder.write_all(&self.digest[..self.kind.output_size()])?;
        Ok(())
    }
}

impl<A: Allocator> DecodeIn<A> for DigestHeader {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(reader), ret, err))]
    #[inline]
    fn decode_in(decoder: &mut impl Decoder, _alloc: A) -> Result<DigestHeader, DecodeError> {
        let kind = decoder.decode()?;
        let mut digest = [0u8; 32];
        decoder.read_exact(&mut digest)?;

        Ok(DigestHeader { kind, digest })
    }
}
