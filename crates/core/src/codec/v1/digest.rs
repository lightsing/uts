use crate::{
    codec::{Decode, Decoder, Encode, Encoder, v1::opcode::DigestOp},
    error::{DecodeError, EncodeError},
    utils::Hexed,
};
use std::fmt;

/// Header describing the digest that anchors a timestamp.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DigestHeader {
    kind: DigestOp,
    digest: [u8; 32],
}

impl fmt::Display for DigestHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.kind,
            Hexed(&self.digest[..self.kind.output_size()])
        )
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
    fn encode(&self, mut writer: impl Encoder) -> Result<(), EncodeError> {
        writer.encode(&self.kind)?;
        writer.write_all(&self.digest[..self.kind.output_size()])?;
        Ok(())
    }
}

impl Decode for DigestHeader {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(reader), ret, err))]
    #[inline]
    fn decode(mut reader: impl Decoder) -> Result<DigestHeader, DecodeError> {
        let kind = reader.decode()?;
        let mut digest = [0u8; 32];
        reader.read_exact(&mut digest)?;

        Ok(DigestHeader { kind, digest })
    }
}
