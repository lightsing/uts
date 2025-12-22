use alloy_primitives::{Keccak256, b256};
use alloy_signer::SignerSync;
use alloy_signer_local::LocalSigner;
use axum::body::Bytes;
use bytes::BytesMut;
use smallvec::SmallVec;
use std::time::SystemTime;
use tracing::Level;
use uts_core::{
    codec::{
        Encoder,
        v1::{Attestation, PendingAttestation, opcode::OpCode},
    },
    utils::Hexed,
};

pub const MAX_DIGEST_SIZE: usize = 64; // e.g., SHA3-512
const ERC2098_SIGNATURE_SIZE: usize = 64;

// Test this with official ots client:
// ots stamp -c "http://localhost:3000/" -m 1 <input_file>
// cargo run --bin uts-info -- <input_file>.ots
// Sample:
// ```
// OTS Detached Timestamp found:
// Version 1 Proof digest of SHA256 877c470874fa92e5609a1396b1188ffa3e539d83ec2748a7cb6fb2d4430d45a2
// execute APPEND ec04517482d3be52b6123ca37f683285
// result 877c470874fa92e5609a1396b1188ffa3e539d83ec2748a7cb6fb2d4430d45a2ec04517482d3be52b6123ca37f683285
// execute SHA256
// result 2edc60a195a879bd446c5473921c46db14c4b1974516682ecae2b406121a5732
// execute PREPEND 5137456900000000
// result 51374569000000002edc60a195a879bd446c5473921c46db14c4b1974516682ecae2b406121a5732
// execute APPEND 9f947a5cf576ba4f68593ac5e350204cc8b38bf0fd5f6f2d4436820d3164dfeaf7405188dfc4bad66e8f42e6fd0a6ffdcceebda548d01224113baab1a568a2b8
// result 51374569000000002edc60a195a879bd446c5473921c46db14c4b1974516682ecae2b406121a57329f947a5cf576ba4f68593ac5e350204cc8b38bf0fd5f6f2d4436820d3164dfeaf7405188dfc4bad66e8f42e6fd0a6ffdcceebda548d01224113baab1a568a2b8
// execute KECCAK256
// result c15b4e8b93e9aaee5b8c736f5b73e5f313062e389925a0b1fc6495053f99d352
// result attested by Pending: update URI https://localhost:3000
// ```
#[instrument(level = Level::TRACE, skip_all)]
pub async fn submit_digest(digest: Bytes) -> Bytes {
    const MAX_MESSAGE_SIZE: usize = MAX_DIGEST_SIZE + size_of::<u64>() + ERC2098_SIGNATURE_SIZE;

    let uri = "https://localhost:3000".to_string();

    let buf_size = 1 // OpCode::PREPEND
        + 1 // length of u64 length in leb128
        + 8 // u64 timestamp
        + 1 // OpCode::APPEND
        + 1 // length of signature length in leb128
        + ERC2098_SIGNATURE_SIZE // signature
        + 1 // FIXME: TBD: OpCode::KECCAK256
        + 1 // OpCode::ATTESTATION
        + 8 // Pending tag
        + 1 // length of packed ATTESTATION data length in leb128
        + (1 + uri.len()); // length of uri in leb128 + uri bytes
    let attestation = PendingAttestation { uri: uri.into() };

    let mut timestamp = BytesMut::with_capacity(buf_size);

    let mut pending_attestation = SmallVec::<[u8; MAX_MESSAGE_SIZE]>::new();

    // ots uses 32-bit unix time, but we use u64 here for future proofing, as it's not part of the ots spec.
    let recv_timestamp: u64 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Clock MUST not go backwards")
        .as_secs();
    trace!(recv_timestamp);
    let recv_timestamp = recv_timestamp.to_le_bytes();
    timestamp.encode(OpCode::PREPEND).unwrap();
    timestamp.encode_bytes(&recv_timestamp).unwrap();
    pending_attestation.extend(recv_timestamp);

    trace!(digest = ?Hexed(&digest));
    pending_attestation.extend_from_slice(&digest);

    let signer = LocalSigner::from_bytes(&b256!(
        "9ba9926331eb5f4995f1e358f57ba1faab8b005b51928d2fdaea16e69a6ad225"
    ))
    .unwrap(); // TODO: load from app state
    let undeniable_sig = signer.sign_message_sync(&digest).unwrap();
    let undeniable_sig = undeniable_sig.as_erc2098();
    trace!(undeniable_sig = ?Hexed(&undeniable_sig));
    timestamp.encode(OpCode::APPEND).unwrap();
    timestamp.encode_bytes(&undeniable_sig).unwrap();
    pending_attestation.extend(undeniable_sig);

    trace!(pending_attestation = ?Hexed(&pending_attestation));

    // FIXME:
    // discussion: return the hash or the raw timestamp message?
    // if using hash, client will request upgrade timestamp by hash (256 bits, 64 hex chars)
    //
    // if using raw timestamp message, client will request timestamp by whole message (variable size, 208 hex chars if request is 32 bytes),
    // but we will have info about the receiving time of the request,
    // which can narrow down the search space
    let mut hasher = Keccak256::new();
    hasher.update(&pending_attestation);
    hasher.finalize_into(&mut pending_attestation[0..32]);
    timestamp.encode(OpCode::KECCAK256).unwrap();

    timestamp.encode(OpCode::ATTESTATION).unwrap();
    timestamp.encode(&attestation.to_raw().unwrap()).unwrap();

    // TODO: store the pending_attestation into journal
    debug_assert_eq!(timestamp.len(), buf_size, "buffer size mismatch");
    timestamp.freeze()
}

pub async fn get_timestamp() {}
