//! # UTS FFI Binding
//!
//! UniFFI-based foreign function interface for the [`uts-core`] library.
//!
//! This crate exposes timestamp encoding, decoding, and inspection
//! functionality to foreign languages (Python, Kotlin, Swift, etc.)
//! using [UniFFI](https://mozilla.github.io/uniffi-rs/) proc macros.
//!
//! # Overview
//!
//! The primary entry point is [`DetachedTimestamp`], which wraps a decoded
//! OpenTimestamps proof and provides methods to inspect and manipulate it.
//!
//! Standalone utility functions are also exported:
//!
//! - [`uts_execute_op`] — execute an opcode on input data
//! - [`uts_validate_pending_uri`] — validate a pending attestation URI
//! - [`uts_digest_output_size`] — query the output size of a digest algorithm
//! - [`uts_opcode_name`] — get the human-readable name of an opcode
//! - [`uts_magic_bytes`] — get the OTS file magic bytes

use std::sync::{Arc, RwLock};

use uts_core::{
    codec::{
        Decode, Encode, VersionedProof,
        v1::{
            Attestation as AttestationTrait, BitcoinAttestation as CoreBitcoinAttestation,
            DetachedTimestamp as CoreDetachedTimestamp, DigestHeader as CoreDigestHeader,
            EASAttestation as CoreEASAttestation, EASTimestamped as CoreEASTimestamped,
            PendingAttestation as CorePendingAttestation, RawAttestation as CoreRawAttestation,
            opcode::{DigestOp as CoreDigestOp, OpCode as CoreOpCode},
        },
    },
    error::{DecodeError, EncodeError},
};

uniffi::setup_scaffolding!();

// ── Error ──────────────────────────────────────────────────────────────────────

/// Errors that can occur during UTS FFI operations.
#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum UtsError {
    /// An error occurred while decoding a proof.
    #[error("decode error: {0}")]
    DecodeError(String),
    /// An error occurred while encoding a proof.
    #[error("encode error: {0}")]
    EncodeError(String),
    /// An invalid operation was attempted.
    #[error("invalid operation: {0}")]
    InvalidOperation(String),
}

impl From<DecodeError> for UtsError {
    fn from(e: DecodeError) -> Self {
        UtsError::DecodeError(e.to_string())
    }
}

impl From<EncodeError> for UtsError {
    fn from(e: EncodeError) -> Self {
        UtsError::EncodeError(e.to_string())
    }
}

// ── FFI Enums ──────────────────────────────────────────────────────────────────

/// Supported digest (hash) algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum DigestOp {
    /// SHA-1 (20-byte output).
    Sha1,
    /// RIPEMD-160 (20-byte output).
    Ripemd160,
    /// SHA-256 (32-byte output).
    Sha256,
    /// Keccak-256 (32-byte output).
    Keccak256,
}

impl From<CoreDigestOp> for DigestOp {
    fn from(op: CoreDigestOp) -> Self {
        match op.tag() {
            0x02 => Self::Sha1,
            0x03 => Self::Ripemd160,
            0x08 => Self::Sha256,
            0x67 => Self::Keccak256,
            _ => unreachable!("invalid digest op tag"),
        }
    }
}

impl From<DigestOp> for CoreDigestOp {
    fn from(op: DigestOp) -> Self {
        match op {
            DigestOp::Sha1 => CoreDigestOp::SHA1,
            DigestOp::Ripemd160 => CoreDigestOp::RIPEMD160,
            DigestOp::Sha256 => CoreDigestOp::SHA256,
            DigestOp::Keccak256 => CoreDigestOp::KECCAK256,
        }
    }
}

/// A parsed attestation from a timestamp proof.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum Attestation {
    /// A Bitcoin block attestation.
    Bitcoin {
        /// Block height at which the attestation was recorded.
        height: u32,
    },
    /// An Ethereum Attestation Service (EAS) attestation.
    Eas {
        /// Chain ID of the blockchain.
        chain_id: u64,
        /// Unique identifier of the attestation (32 bytes).
        uid: Vec<u8>,
    },
    /// An EAS timestamped attestation.
    EasTimestamped {
        /// Chain ID of the blockchain.
        chain_id: u64,
    },
    /// A pending attestation (not yet confirmed on-chain).
    Pending {
        /// URI where the upgraded proof can be fetched.
        uri: String,
    },
    /// An attestation type not recognized by this library.
    Unknown {
        /// 8-byte tag identifying the attestation type.
        tag: Vec<u8>,
        /// Raw attestation payload.
        data: Vec<u8>,
    },
}

// ── FFI Records ────────────────────────────────────────────────────────────────

/// Header describing the digest that anchors a timestamp.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct DigestHeader {
    /// The digest algorithm used.
    pub kind: DigestOp,
    /// The digest bytes, trimmed to the algorithm's output size.
    pub digest: Vec<u8>,
}

impl From<&CoreDigestHeader> for DigestHeader {
    fn from(header: &CoreDigestHeader) -> Self {
        DigestHeader {
            kind: header.kind().into(),
            digest: header.digest().to_vec(),
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Attempts to parse a [`CoreRawAttestation`] into a high-level [`Attestation`].
fn parse_raw_attestation(raw: &CoreRawAttestation) -> Attestation {
    if let Ok(att) = CoreBitcoinAttestation::from_raw(raw) {
        return Attestation::Bitcoin { height: att.height };
    }
    if let Ok(att) = CoreEASAttestation::from_raw(raw) {
        return Attestation::Eas {
            chain_id: att.chain.id(),
            uid: att.uid.as_slice().to_vec(),
        };
    }
    if let Ok(att) = CoreEASTimestamped::from_raw(raw) {
        return Attestation::EasTimestamped {
            chain_id: att.chain.id(),
        };
    }
    if let Ok(att) = CorePendingAttestation::from_raw(raw) {
        return Attestation::Pending {
            uri: att.uri.into_owned(),
        };
    }
    Attestation::Unknown {
        tag: raw.tag.as_slice().to_vec(),
        data: raw.data.as_slice().to_vec(),
    }
}

// ── Main Object ────────────────────────────────────────────────────────────────

/// An opaque handle to a decoded detached timestamp proof.
///
/// Wraps a versioned `DetachedTimestamp` from `uts-core` and exposes methods
/// for inspection, encoding, and mutation through the FFI boundary.
#[derive(Debug, uniffi::Object)]
pub struct DetachedTimestamp {
    inner: RwLock<VersionedProof<CoreDetachedTimestamp>>,
}

#[uniffi::export]
impl DetachedTimestamp {
    /// Decodes a detached timestamp from its binary OTS representation.
    ///
    /// The input must be a complete OTS file including magic bytes and version.
    #[uniffi::constructor]
    pub fn new(data: Vec<u8>) -> Result<Arc<Self>, UtsError> {
        let mut reader = data.as_slice();
        let proof = VersionedProof::<CoreDetachedTimestamp>::decode(&mut reader)?;
        Ok(Arc::new(DetachedTimestamp {
            inner: RwLock::new(proof),
        }))
    }

    /// Encodes the detached timestamp back to its binary OTS representation.
    ///
    /// Returns the complete OTS file bytes including magic bytes and version.
    pub fn encode(&self) -> Result<Vec<u8>, UtsError> {
        let guard = self
            .inner
            .read()
            .map_err(|e| UtsError::InvalidOperation(format!("lock poisoned: {e}")))?;
        let mut buf = Vec::new();
        Encode::encode(&*guard, &mut buf)?;
        Ok(buf)
    }

    /// Returns the digest header of this timestamp.
    pub fn header(&self) -> Result<DigestHeader, UtsError> {
        let guard = self
            .inner
            .read()
            .map_err(|e| UtsError::InvalidOperation(format!("lock poisoned: {e}")))?;
        Ok(DigestHeader::from(guard.proof().header()))
    }

    /// Returns all attestations in the timestamp tree.
    ///
    /// Each raw attestation is parsed into its typed representation.
    /// Unknown attestation types are returned as [`Attestation::Unknown`].
    pub fn attestations(&self) -> Result<Vec<Attestation>, UtsError> {
        let guard = self
            .inner
            .read()
            .map_err(|e| UtsError::InvalidOperation(format!("lock poisoned: {e}")))?;
        Ok(guard
            .proof()
            .timestamp()
            .attestations()
            .map(parse_raw_attestation)
            .collect())
    }

    /// Returns `true` if the timestamp has been finalized.
    pub fn is_finalized(&self) -> Result<bool, UtsError> {
        let guard = self
            .inner
            .read()
            .map_err(|e| UtsError::InvalidOperation(format!("lock poisoned: {e}")))?;
        Ok(guard.proof().timestamp().is_finalized())
    }

    /// Returns a human-readable representation of the timestamp.
    pub fn display(&self) -> Result<String, UtsError> {
        let guard = self
            .inner
            .read()
            .map_err(|e| UtsError::InvalidOperation(format!("lock poisoned: {e}")))?;
        Ok(format!("{}", guard.proof()))
    }

    /// Removes all pending attestations from the timestamp tree.
    ///
    /// Returns the number of pending attestations removed, or `None` if the
    /// entire timestamp consisted only of pending attestations (making it empty).
    pub fn purge_pending(&self) -> Result<Option<u32>, UtsError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|e| UtsError::InvalidOperation(format!("lock poisoned: {e}")))?;
        let result = guard.proof.purge_pending();
        Ok(result.map(|n| n as u32))
    }
}

// ── Free Functions ─────────────────────────────────────────────────────────────

/// Executes an opcode on the given input data with an optional immediate value.
///
/// `op_tag` is the raw byte identifying the opcode (e.g. `0x08` for SHA-256).
/// Control opcodes (`ATTESTATION` = 0x00, `FORK` = 0xff) are not executable.
#[uniffi::export]
pub fn uts_execute_op(op_tag: u8, input: Vec<u8>, immediate: Vec<u8>) -> Result<Vec<u8>, UtsError> {
    let op = CoreOpCode::new(op_tag)
        .ok_or_else(|| UtsError::InvalidOperation(format!("unknown opcode: 0x{op_tag:02x}")))?;
    if op.is_control() {
        return Err(UtsError::InvalidOperation(format!(
            "cannot execute control opcode: {}",
            op.name()
        )));
    }
    Ok(op.execute(&input, &immediate).to_vec())
}

/// Validates a URI for use in a pending attestation.
///
/// Returns `true` if the URI contains only allowed characters
/// (`a-z`, `A-Z`, `0-9`, `.`, `-`, `_`, `/`, `:`) and does not exceed
/// the maximum length of 1000 bytes.
#[uniffi::export]
pub fn uts_validate_pending_uri(uri: String) -> bool {
    uri.len() <= CorePendingAttestation::MAX_URI_LEN && CorePendingAttestation::validate_uri(&uri)
}

/// Returns the output size in bytes for the given digest algorithm.
#[uniffi::export]
pub fn uts_digest_output_size(op: DigestOp) -> u32 {
    CoreDigestOp::from(op).output_size() as u32
}

/// Returns the human-readable name of an opcode given its byte tag.
///
/// Returns an error if the tag does not correspond to a known opcode.
#[uniffi::export]
pub fn uts_opcode_name(op_tag: u8) -> Result<String, UtsError> {
    let op = CoreOpCode::new(op_tag)
        .ok_or_else(|| UtsError::InvalidOperation(format!("unknown opcode: 0x{op_tag:02x}")))?;
    Ok(op.name().to_owned())
}

/// Returns the OTS file magic bytes.
///
/// Every valid OTS file begins with these 31 bytes.
#[uniffi::export]
pub fn uts_magic_bytes() -> Vec<u8> {
    uts_core::codec::MAGIC.to_vec()
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Embedded copy of the small detached timestamp fixture from uts-core.
    const SMALL_OTS: &[u8] = b"\
\x00\x4f\x70\x65\x6e\x54\x69\x6d\x65\x73\x74\x61\x6d\x70\x73\x00\x00\x50\x72\x6f\x6f\x66\x00\xbf\x89\xe2\xe8\x84\xe8\x92\
\x94\x01\x08\xa7\x0d\xfe\x69\xc5\xa0\xd6\x28\x16\x78\x1a\xbb\x6e\x17\x77\x85\x47\x18\x62\x4a\x0d\x19\x42\x31\xad\xb1\x4c\
\x32\xee\x54\x38\xa4\xf0\x10\x7a\x46\x05\xde\x0a\x5b\x37\xcb\x21\x17\x59\xc6\x81\x2b\xfe\x2e\x08\xff\xf0\x10\x24\x4b\x79\
\xd5\x78\xaa\x38\xe3\x4f\x42\x7b\x0f\x3e\xd2\x55\xa5\x08\xf1\x04\x58\xa4\xc2\x57\xf0\x08\xa1\xa9\x2c\x61\xd5\x41\x72\x06\
\x00\x83\xdf\xe3\x0d\x2e\xf9\x0c\x8e\x2c\x2b\x68\x74\x74\x70\x73\x3a\x2f\x2f\x62\x6f\x62\x2e\x62\x74\x63\x2e\x63\x61\x6c\
\x65\x6e\x64\x61\x72\x2e\x6f\x70\x65\x6e\x74\x69\x6d\x65\x73\x74\x61\x6d\x70\x73\x2e\x6f\x72\x67\xf0\x10\xe0\x27\x85\x91\
\xe2\x88\x68\x19\xba\x7b\x3d\xdd\x63\x2e\xd3\xfe\x08\xf1\x04\x58\xa4\xc2\x56\xf0\x08\x38\xf2\xc7\xf4\xba\xf4\xbc\xd7\x00\
\x83\xdf\xe3\x0d\x2e\xf9\x0c\x8e\x2e\x2d\x68\x74\x74\x70\x73\x3a\x2f\x2f\x61\x6c\x69\x63\x65\x2e\x62\x74\x63\x2e\x63\x61\
\x6c\x65\x6e\x64\x61\x72\x2e\x6f\x70\x65\x6e\x74\x69\x6d\x65\x73\x74\x61\x6d\x70\x73\x2e\x6f\x72\x67";

    #[test]
    fn decode_and_inspect() {
        let ts = DetachedTimestamp::new(SMALL_OTS.to_vec()).unwrap();

        let header = ts.header().unwrap();
        assert_eq!(header.kind, DigestOp::Sha256);
        assert_eq!(header.digest.len(), 32);

        assert!(ts.is_finalized().unwrap());
    }

    #[test]
    fn decode_attestations() {
        let ts = DetachedTimestamp::new(SMALL_OTS.to_vec()).unwrap();
        let atts = ts.attestations().unwrap();

        // The small fixture has 2 pending attestations.
        assert_eq!(atts.len(), 2);
        for att in &atts {
            match att {
                Attestation::Pending { uri } => {
                    assert!(uri.starts_with("https://"));
                    assert!(uri.contains("calendar.opentimestamps.org"));
                }
                other => panic!("expected Pending attestation, got {other:?}"),
            }
        }
    }

    #[test]
    fn round_trip_encode_decode() {
        let ts = DetachedTimestamp::new(SMALL_OTS.to_vec()).unwrap();
        let encoded = ts.encode().unwrap();
        assert_eq!(encoded, SMALL_OTS);

        // Re-decode to make sure it's still valid.
        let ts2 = DetachedTimestamp::new(encoded).unwrap();
        assert_eq!(ts.header().unwrap(), ts2.header().unwrap());
        assert_eq!(ts.attestations().unwrap(), ts2.attestations().unwrap());
    }

    #[test]
    fn purge_pending_removes_all() {
        let ts = DetachedTimestamp::new(SMALL_OTS.to_vec()).unwrap();

        // All attestations are pending, so purge returns None (empty tree).
        let result = ts.purge_pending().unwrap();
        assert!(result.is_none(), "all-pending purge should return None");
    }

    #[test]
    fn display_produces_output() {
        let ts = DetachedTimestamp::new(SMALL_OTS.to_vec()).unwrap();
        let text = ts.display().unwrap();
        assert!(!text.is_empty());
        assert!(text.contains("SHA256"));
    }

    #[test]
    fn decode_invalid_data() {
        let result = DetachedTimestamp::new(vec![0x00, 0x01, 0x02]);
        assert!(result.is_err());
    }

    #[test]
    fn execute_sha256() {
        let result = uts_execute_op(0x08, b"hello".to_vec(), vec![]).unwrap();
        assert_eq!(result.len(), 32);

        // Verify it matches the known SHA-256 of "hello".
        let expected = sha2::Digest::finalize(sha2::Digest::chain_update(
            sha2::Sha256::default(),
            b"hello",
        ));
        assert_eq!(result, expected.as_slice());
    }

    #[test]
    fn execute_append() {
        let result = uts_execute_op(0xf0, b"hello".to_vec(), b" world".to_vec()).unwrap();
        assert_eq!(result, b"hello world");
    }

    #[test]
    fn execute_prepend() {
        let result = uts_execute_op(0xf1, b"world".to_vec(), b"hello ".to_vec()).unwrap();
        assert_eq!(result, b"hello world");
    }

    #[test]
    fn execute_reverse() {
        let result = uts_execute_op(0xf2, b"abcd".to_vec(), vec![]).unwrap();
        assert_eq!(result, b"dcba");
    }

    #[test]
    fn execute_hexlify() {
        let result = uts_execute_op(0xf3, vec![0xde, 0xad], vec![]).unwrap();
        assert_eq!(result, b"dead");
    }

    #[test]
    fn execute_control_opcode_errors() {
        // ATTESTATION (0x00) is a control opcode.
        assert!(uts_execute_op(0x00, vec![], vec![]).is_err());
        // FORK (0xff) is a control opcode.
        assert!(uts_execute_op(0xff, vec![], vec![]).is_err());
    }

    #[test]
    fn execute_unknown_opcode_errors() {
        assert!(uts_execute_op(0x42, vec![], vec![]).is_err());
    }

    #[test]
    fn validate_pending_uri_valid() {
        assert!(uts_validate_pending_uri(
            "https://bob.btc.calendar.opentimestamps.org".to_owned()
        ));
    }

    #[test]
    fn validate_pending_uri_invalid_char() {
        assert!(!uts_validate_pending_uri(
            "https://example.com/path?q=1".to_owned()
        ));
    }

    #[test]
    fn validate_pending_uri_too_long() {
        let long_uri = "a".repeat(1001);
        assert!(!uts_validate_pending_uri(long_uri));
    }

    #[test]
    fn digest_output_sizes() {
        assert_eq!(uts_digest_output_size(DigestOp::Sha1), 20);
        assert_eq!(uts_digest_output_size(DigestOp::Ripemd160), 20);
        assert_eq!(uts_digest_output_size(DigestOp::Sha256), 32);
        assert_eq!(uts_digest_output_size(DigestOp::Keccak256), 32);
    }

    #[test]
    fn opcode_names() {
        assert_eq!(uts_opcode_name(0x08).unwrap(), "SHA256");
        assert_eq!(uts_opcode_name(0xf0).unwrap(), "APPEND");
        assert_eq!(uts_opcode_name(0xff).unwrap(), "FORK");
        assert!(uts_opcode_name(0x42).is_err());
    }

    #[test]
    fn magic_bytes() {
        let magic = uts_magic_bytes();
        assert_eq!(magic.len(), 31);
        assert_eq!(&magic[1..15], b"OpenTimestamps");
    }

    use sha2::Digest as _;

    #[test]
    fn build_and_round_trip() {
        use uts_core::codec::v1::{
            BitcoinAttestation as CoreBitcoinAttestation,
            DetachedTimestamp as CoreDetachedTimestamp, DigestHeader as CoreDigestHeader,
            Timestamp as CoreTimestamp,
        };

        // Build a simple timestamp: SHA256 digest → Bitcoin attestation.
        let digest_output = sha2::Sha256::digest(b"test data");
        let header = CoreDigestHeader::new::<sha2::Sha256>(digest_output);

        let mut builder = CoreTimestamp::builder();
        builder.sha256();
        let timestamp = builder
            .attest(CoreBitcoinAttestation { height: 840_000 })
            .unwrap();

        let detached = CoreDetachedTimestamp::from_parts(header, timestamp);
        let versioned = VersionedProof::new(detached);

        let mut raw_bytes = Vec::new();
        Encode::encode(&versioned, &mut raw_bytes).unwrap();

        // Decode through FFI.
        let ts = DetachedTimestamp::new(raw_bytes.clone()).unwrap();

        let h = ts.header().unwrap();
        assert_eq!(h.kind, DigestOp::Sha256);
        assert_eq!(h.digest, digest_output.as_slice());

        let atts = ts.attestations().unwrap();
        assert_eq!(atts.len(), 1);
        assert_eq!(atts[0], Attestation::Bitcoin { height: 840_000 });

        // Round-trip encode.
        let re_encoded = ts.encode().unwrap();
        assert_eq!(re_encoded, raw_bytes);
    }
}
