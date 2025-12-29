//! # Attestations
//!
//! An attestation is a claim that some data existed at some time. It
//! comes from some server or from a blockchain.

use crate::{
    codec::{Decode, DecodeIn, Decoder, Encode, Encoder, v1::MayHaveInput},
    error::{DecodeError, EncodeError},
    utils::{Hexed, OnceLock},
};
use alloc::{
    alloc::{Allocator, Global},
    borrow::Cow,
    vec::Vec,
};
use core::fmt;

/// Size in bytes of the tag identifying the attestation type.
const TAG_SIZE: usize = 8;

/// Tag indicating a Bitcoin attestation.
const BITCOIN_TAG: &[u8; 8] = b"\x05\x88\x96\x0d\x73\xd7\x19\x01";
/// Tag indicating a pending attestation.
const PENDING_TAG: &[u8; 8] = b"\x83\xdf\xe3\x0d\x2e\xf9\x0c\x8e";

/// Tag identifying the attestation kind.
pub type AttestationTag = [u8; TAG_SIZE];

/// Raw Proof that some data existed at a given time.
#[derive(Clone)]
pub struct RawAttestation<A: Allocator = Global> {
    pub tag: AttestationTag,
    pub data: Vec<u8, A>,
    /// Cached value for verifying the attestation.
    pub(crate) value: OnceLock<Vec<u8, A>>,
}

impl<A: Allocator> fmt::Debug for RawAttestation<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawAttestation")
            .field("tag", &Hexed(&self.tag))
            .field("data", &Hexed(&self.data))
            .finish()
    }
}

impl<A: Allocator> DecodeIn<A> for RawAttestation<A> {
    fn decode_in(decoder: &mut impl Decoder, alloc: A) -> Result<Self, DecodeError> {
        let mut tag = [0u8; TAG_SIZE];
        decoder.read_exact(&mut tag)?;

        let len = decoder.decode()?;
        let mut data = Vec::with_capacity_in(len, alloc);
        data.resize(len, 0);
        decoder.read_exact(&mut data)?;

        Ok(RawAttestation {
            tag,
            data,
            value: OnceLock::new(),
        })
    }
}

impl<A: Allocator> Encode for RawAttestation<A> {
    #[inline]
    fn encode(&self, encoder: &mut impl Encoder) -> Result<(), EncodeError> {
        encoder.write_all(self.tag)?;
        encoder.encode_bytes(&self.data)
    }
}

impl<A: Allocator> PartialEq for RawAttestation<A> {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag && self.data.as_slice() == other.data.as_slice()
    }
}

impl<A: Allocator> Eq for RawAttestation<A> {}

impl<A: Allocator> RawAttestation<A> {
    /// Returns the allocator used by this raw attestation.
    #[inline]
    pub fn allocator(&self) -> &A {
        self.data.allocator()
    }
}

pub trait Attestation<'a>: Sized {
    const TAG: AttestationTag;

    fn from_raw<A: Allocator>(raw: &'a RawAttestation<A>) -> Result<Self, DecodeError> {
        if raw.tag != Self::TAG {
            return Err(DecodeError::BadAttestationTag);
        }

        Self::from_raw_data(&raw.data)
    }

    fn to_raw(&self) -> Result<RawAttestation, EncodeError> {
        self.to_raw_in(Global)
    }

    fn to_raw_in<A: Allocator>(&self, alloc: A) -> Result<RawAttestation<A>, EncodeError> {
        Ok(RawAttestation {
            tag: Self::TAG,
            data: self.to_raw_data_in(alloc)?,
            value: OnceLock::new(),
        })
    }

    fn from_raw_data(data: &'a [u8]) -> Result<Self, DecodeError>;
    fn to_raw_data_in<A: Allocator>(&self, alloc: A) -> Result<Vec<u8, A>, EncodeError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitcoinAttestation {
    pub height: u32,
}

impl fmt::Display for BitcoinAttestation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bitcoin at height {}", self.height)
    }
}

impl Attestation<'_> for BitcoinAttestation {
    const TAG: AttestationTag = *BITCOIN_TAG;

    fn from_raw_data(data: &[u8]) -> Result<Self, DecodeError> {
        let height = u32::decode(&mut &*data)?;
        Ok(BitcoinAttestation { height })
    }

    fn to_raw_data_in<A: Allocator>(&self, alloc: A) -> Result<Vec<u8, A>, EncodeError> {
        let mut buffer = Vec::with_capacity_in(u32::BITS.div_ceil(7) as usize, alloc);
        buffer.encode(self.height)?;
        Ok(buffer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingAttestation<'a> {
    pub uri: Cow<'a, str>,
}

impl PendingAttestation<'_> {
    /// Maximum length of a URI in a "pending" attestation.
    pub const MAX_URI_LEN: usize = 1000;

    #[inline]
    pub fn validate_uri(uri: &str) -> bool {
        uri.chars()
            .all(|ch| matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' | '/' | ':'))
    }
}

impl fmt::Display for PendingAttestation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pending at {}", self.uri)
    }
}

impl<'a> Attestation<'a> for PendingAttestation<'a> {
    const TAG: AttestationTag = *PENDING_TAG;

    fn from_raw_data(data: &'a [u8]) -> Result<Self, DecodeError> {
        let data = &mut &data[..];
        let length = u32::decode(data)? as usize; // length prefix
        if length > Self::MAX_URI_LEN {
            return Err(DecodeError::UriTooLong);
        }
        if data.len() < length {
            return Err(DecodeError::UnexpectedEof);
        }
        let uri = core::str::from_utf8(&data[..length]).map_err(|_| DecodeError::InvalidUriChar)?;
        if !Self::validate_uri(uri) {
            return Err(DecodeError::InvalidUriChar);
        }
        Ok(PendingAttestation {
            uri: Cow::Borrowed(uri),
        })
    }

    fn to_raw_data_in<A: Allocator>(&self, alloc: A) -> Result<Vec<u8, A>, EncodeError> {
        if self.uri.len() > Self::MAX_URI_LEN {
            return Err(EncodeError::UriTooLong);
        }
        if !Self::validate_uri(&self.uri) {
            return Err(EncodeError::InvalidUriChar);
        }
        let mut buffer =
            Vec::with_capacity_in(self.uri.len() + u32::BITS.div_ceil(7) as usize, alloc);
        buffer.encode_bytes(self.uri.as_bytes())?;
        Ok(buffer)
    }
}

impl<A: Allocator> fmt::Display for RawAttestation<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.tag {
            tag if *tag == *BITCOIN_TAG => {
                let att = BitcoinAttestation::from_raw(self).expect("Valid Bitcoin attestation");
                write!(f, "{}", att)
            }
            tag if *tag == *PENDING_TAG => {
                let att = PendingAttestation::from_raw(self).expect("Valid Pending attestation");
                write!(f, "{}", att)
            }
            _ => write!(f, "Unknown Attestation with tag {}", Hexed(&self.tag)),
        }
    }
}

impl<A: Allocator> MayHaveInput for RawAttestation<A> {
    #[inline]
    fn input(&self) -> Option<&[u8]> {
        self.value.get().map(|v| v.as_slice())
    }
}
