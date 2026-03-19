use crate::UtsError;
use alloy_primitives::B256;
use uniffi::{Enum, Record};
use uts_core::codec::v1::{self as codec_v1, Attestation as _, AttestationTag, RawAttestation};

#[derive(Debug, Clone, PartialEq, Eq, Enum)]
#[non_exhaustive]
pub enum Attestation {
    Bitcoin(BitcoinAttestation),
    EASAttestation(EASAttestation),
    EASTimestamped(EASTimestamped),
    Pending(PendingAttestation),
    Unknown(UnknownAttestation),
}

#[derive(Debug, Clone, PartialEq, Eq, Record)]
pub struct BitcoinAttestation {
    pub height: u32,
}

impl From<codec_v1::BitcoinAttestation> for BitcoinAttestation {
    fn from(attestation: codec_v1::BitcoinAttestation) -> Self {
        Self {
            height: attestation.height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Record)]
pub struct EASAttestation {
    pub chain: u64,
    pub uid: B256,
}

impl From<codec_v1::EASAttestation> for EASAttestation {
    fn from(attestation: codec_v1::EASAttestation) -> Self {
        Self {
            chain: attestation.chain.id(),
            uid: attestation.uid,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Record)]
pub struct EASTimestamped {
    pub chain: u64,
}

impl From<codec_v1::EASTimestamped> for EASTimestamped {
    fn from(attestation: codec_v1::EASTimestamped) -> Self {
        Self {
            chain: attestation.chain.id(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Record)]
pub struct PendingAttestation {
    pub uri: String,
}

impl From<codec_v1::PendingAttestation<'_>> for PendingAttestation {
    fn from(attestation: codec_v1::PendingAttestation<'_>) -> Self {
        Self {
            uri: attestation.uri.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Record)]
pub struct UnknownAttestation {
    pub tag: AttestationTag,
    pub value: Vec<u8>,
}

impl TryFrom<&RawAttestation> for Attestation {
    type Error = UtsError;

    fn try_from(attestation: &RawAttestation) -> Result<Self, Self::Error> {
        macro_rules! try_decode_as {
            ($kind:tt, $variant:tt) => {
                if attestation.tag == codec_v1::$kind::TAG {
                    let attestation = codec_v1::$kind::from_raw(&attestation)?;
                    let attestation = $kind::from(attestation);
                    return Ok(Attestation::$variant(attestation));
                }
            };
        }

        try_decode_as!(BitcoinAttestation, Bitcoin);
        try_decode_as!(EASAttestation, EASAttestation);
        try_decode_as!(EASTimestamped, EASTimestamped);
        try_decode_as!(PendingAttestation, Pending);

        Ok(Attestation::Unknown(UnknownAttestation {
            tag: attestation.tag,
            value: attestation.data.to_vec(),
        }))
    }
}
