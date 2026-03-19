use crate::{UtsError, codec::v1::DigestOp};
use uniffi::Record;
use uts_core::codec::v1 as codec_v1;

#[derive(Debug, Clone, PartialEq, Eq, Record)]
pub struct DigestHeader {
    kind: DigestOp,
    digest: Vec<u8>,
}

impl TryFrom<DigestHeader> for codec_v1::DigestHeader {
    type Error = UtsError;

    fn try_from(header: DigestHeader) -> Result<Self, Self::Error> {
        let kind = codec_v1::opcode::DigestOp::from(header.kind);
        if header.digest.len() != kind.output_size() {
            return Err(UtsError::InvalidDigestLength);
        }
        Ok(codec_v1::DigestHeader::from_slice_unchecked(
            kind,
            &header.digest,
        ))
    }
}

impl TryFrom<&codec_v1::DigestHeader> for DigestHeader {
    type Error = UtsError;

    fn try_from(header: &codec_v1::DigestHeader) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: header.kind().try_into()?,
            digest: header.digest().to_vec(),
        })
    }
}
