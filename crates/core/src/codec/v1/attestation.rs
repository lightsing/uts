// Copyright (C) The OpenTimestamps developers
// Copyright (C) The ots-rs developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Attestations
//!
//! An attestation is a claim that some data existed at some time. It
//! comes from some server or from a blockchain.

use crate::{
    codec::{Decoder, Encoder},
    error::{DecodeError, EncodeError},
    utils::Hexed,
};
use smallvec::SmallVec;
use std::{
    fmt,
    io::{BufRead, Write},
};

/// Size in bytes of the tag identifying the attestation type.
const TAG_SIZE: usize = 8;
/// Maximum length of a URI in a "pending" attestation.
const MAX_URI_LEN: usize = 1000;

/// Tag indicating a Bitcoin attestation.
const BITCOIN_TAG: &[u8; 8] = b"\x05\x88\x96\x0d\x73\xd7\x19\x01";
/// Tag indicating a pending attestation.
const PENDING_TAG: &[u8; 8] = b"\x83\xdf\xe3\x0d\x2e\xf9\x0c\x8e";

/// Tag identifying the attestation kind.
pub type AttestationTag = [u8; TAG_SIZE];

/// Proof that some data existed at a given time.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Attestation {
    /// Attestation derived from a Bitcoin block header.
    ///
    /// This consists solely of a block height and asserts that the
    /// current hash matches the Merkle root of the block at that height.
    Bitcoin { height: u32 },
    /// Attestation delivered by an OpenTimestamps calendar server.
    ///
    /// Only a restricted URI is stored locally so that the server can be
    /// queried later for the full proof material.
    Pending { uri: String },
    /// Opaque attestation stored verbatim.
    Unknown { tag: AttestationTag, data: Vec<u8> },
}

impl Attestation {
    /// Decodes an attestation payload from the reader.
    pub fn decode<R: BufRead>(mut reader: R) -> Result<Attestation, DecodeError> {
        let mut tag = [0u8; TAG_SIZE];
        reader.read_exact(&mut tag)?;
        let len = reader.decode()?;

        if tag == *BITCOIN_TAG {
            let height = reader.decode()?;
            Ok(Attestation::Bitcoin { height })
        } else if tag == *PENDING_TAG {
            // This validation logic copied from python-opentimestamps. Peter comments
            // that he is deliberately avoiding ?, &, @, etc., to "keep us out of trouble"
            let length = reader.decode_ranged(0..=MAX_URI_LEN)?;
            let mut uri_bytes = Vec::with_capacity(len);
            uri_bytes.resize(length, 0);
            reader.read_exact(&mut uri_bytes)?;
            let uri_string =
                String::from_utf8(uri_bytes).map_err(|_| DecodeError::InvalidUriChar)?;
            if !uri_string.chars().all(
                |ch| matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' | '/' | ':'),
            ) {
                return Err(DecodeError::InvalidUriChar);
            }

            Ok(Attestation::Pending { uri: uri_string })
        } else {
            let mut data = Vec::with_capacity(len);
            data.resize(len, 0);
            reader.read_exact(&mut data)?;

            Ok(Attestation::Unknown { tag, data })
        }
    }

    /// Encodes the attestation to the writer.
    pub fn encode<W: Write>(&self, mut writer: W) -> Result<(), EncodeError> {
        match *self {
            Attestation::Bitcoin { height } => {
                writer.write_all(BITCOIN_TAG)?;
                let mut buffer = SmallVec::<[u8; u32::BITS.div_ceil(7) as usize]>::new();
                buffer.encode(height)?;
                writer.encode_bytes(&buffer)
            }
            Attestation::Pending { ref uri } => {
                writer.write_all(PENDING_TAG)?;
                let mut buffer = Vec::new();
                buffer.encode_bytes(uri.as_bytes())?;
                writer.encode_bytes(&buffer)
            }
            Attestation::Unknown { ref tag, ref data } => {
                writer.write_all(tag)?;
                writer.encode_bytes(data)
            }
        }
    }
}

impl fmt::Display for Attestation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Attestation::Bitcoin { height } => write!(f, "Bitcoin block {}", height),
            Attestation::Pending { ref uri } => write!(f, "Pending: update URI {}", uri),
            Attestation::Unknown { ref tag, ref data } => write!(
                f,
                "unknown attestation type {}: {}",
                Hexed(tag),
                Hexed(data)
            ),
        }
    }
}
