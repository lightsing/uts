use crate::{
    alloc::Allocator,
    codec::{
        DecodeIn, Decoder, Encode, Encoder,
        v1::opcode::{DigestOp, DigestOpExt},
    },
    error::{DecodeError, EncodeError},
    utils::Hexed,
};
use core::fmt;
use digest::{Output, typenum::Unsigned};

/// Header describing the digest that anchors a timestamp.
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    serde_with::serde_as,
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DigestHeader {
    pub(crate) kind: DigestOp,
    #[cfg_attr(feature = "serde", serde_as(as = "serde_with::hex::Hex"))]
    pub(crate) digest: [u8; 32],
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
    /// Creates a new digest header from the given digest output.
    pub fn new<D: DigestOpExt>(digest: Output<D>) -> Self {
        let mut digest_bytes = [0u8; 32];
        digest_bytes[..D::OutputSize::USIZE].copy_from_slice(&digest);
        DigestHeader {
            kind: D::OPCODE,
            digest: digest_bytes,
        }
    }

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
    #[tracing::instrument(skip_all, err)]
    #[inline]
    fn encode(&self, encoder: &mut impl Encoder) -> Result<(), EncodeError> {
        encoder.encode(self.kind)?;
        encoder.write_all(&self.digest[..self.kind.output_size()])?;
        Ok(())
    }
}

impl<A: Allocator> DecodeIn<A> for DigestHeader {
    #[tracing::instrument(skip_all, ret(level = trace), err)]
    #[inline]
    fn decode_in(decoder: &mut impl Decoder, _alloc: A) -> Result<DigestHeader, DecodeError> {
        let kind = decoder.decode()?;
        let mut digest = [0u8; 32];
        decoder.read_exact(&mut digest)?;

        Ok(DigestHeader { kind, digest })
    }
}
