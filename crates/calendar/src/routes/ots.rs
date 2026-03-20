use crate::{
    AppState,
    routes::{headers::*, responses::*},
    time::current_time_sec,
};
use alloy_chains::Chain;
use alloy_primitives::B256;
use alloy_signer::SignerSync;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, header},
    response::{IntoResponse, Response},
};
use bump_scope::Bump;
use bytes::BytesMut;
use digest::{Digest, Output};
use sha3::Keccak256;
use std::{cell::RefCell, sync::Arc};
use uts_core::{
    alloc::SliceExt,
    codec::{
        Encode,
        v1::{EASTimestamped, PendingAttestation, Timestamp},
    },
};
use uts_journal::Error;
use uts_stamper::{kv::DbExt, sql, sql::AttestationResult};

/// Maximum digest size accepted by the endpoint.
pub const MAX_DIGEST_SIZE: usize = 64; // e.g., SHA3-512

/// Submit digest to calendar server and get pending timestamp in response.
pub async fn submit_digest(State(state): State<Arc<AppState>>, digest: Bytes) -> Response {
    let (output, commitment) = submit_digest_inner(digest, &state.signer, &state.config.server.uri);
    match state.journal.try_commit(&commitment) {
        // journal is full
        Err(Error::Full) => return service_unavailable(),
        // journal is in fatal error status
        Err(Error::Fatal) => return internal_server_error(),
        Ok(()) => {}
    }
    output.into_response()
}

// TODO: We need to benchmark this.
/// inner function to submit digest, returns (encoded timestamp, commitment)
pub fn submit_digest_inner(
    digest: Bytes,
    signer: impl SignerSync,
    pending_uri: &str,
) -> (Bytes, [u8; 32]) {
    const PRE_ALLOCATION_SIZE_HINT: usize = 4096;
    thread_local! {
        // We don't have `.await` in this function, so it's safe to borrow thread local.
        static BUMP: RefCell<Bump> = RefCell::new(Bump::with_size(PRE_ALLOCATION_SIZE_HINT));
        static HASHER: RefCell<Keccak256> = RefCell::new(Keccak256::new());
    }

    // ots uses 32-bit unix time, but we use u64 here for future proofing, as it's not part of the ots spec.
    let recv_timestamp = current_time_sec().to_le_bytes();

    let undeniable_sig = {
        // sign_message_sync invokes heap allocation, so manually hash it.
        const EIP191_PREFIX: &str = "\x19Ethereum Signed Message:\n";
        let hash = HASHER.with(|hasher| {
            let mut hasher = hasher.borrow_mut();
            hasher.update(EIP191_PREFIX.as_bytes());
            match digest.len() {
                // 32 + 8
                32 => hasher.update(b"40"),
                // 64 + 8
                64 => hasher.update(b"72"),
                _ => {
                    let length = digest.len() + size_of::<u64>();
                    let mut buffer = itoa::Buffer::new();
                    let printed = buffer.format(length);
                    hasher.update(printed.as_bytes());
                }
            }
            hasher.update(recv_timestamp);
            hasher.update(&digest);
            hasher.finalize_reset()
        });

        let hash = B256::from_slice(&hash);
        let undeniable_sig = signer.sign_hash_sync(&hash).unwrap();
        undeniable_sig.as_erc2098()
    };

    #[cfg(any(debug_assertions, not(feature = "performance")))]
    trace!(
        recv_timestamp = ?uts_core::utils::Hexed(&recv_timestamp),
        digest = ?uts_core::utils::Hexed(&digest),
        undeniable_sig = ?uts_core::utils::Hexed(&undeniable_sig),
    );

    BUMP.with(|bump| {
        let mut bump = bump.borrow_mut();
        bump.reset();

        let mut builder = Timestamp::builder_in(&*bump);
        builder
            .prepend(SliceExt::to_vec_in(recv_timestamp.as_slice(), &bump))
            .append(SliceExt::to_vec_in(undeniable_sig.as_slice(), &bump))
            .keccak256();

        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(&builder.commitment(&digest));

        let timestamp = builder
            .attest(PendingAttestation {
                uri: pending_uri.into(),
            })
            .unwrap();

        // copy data out of bump
        let mut buf = BytesMut::with_capacity(128);
        timestamp.encode(&mut buf).unwrap();

        #[cfg(any(debug_assertions, not(feature = "performance")))]
        trace!(encoded_length = buf.len(), timestamp = ?timestamp);

        (buf.freeze(), commitment)
    })
}

/// Get current timestamp from calendar server.
pub async fn get_timestamp(
    State(state): State<Arc<AppState>>,
    Path(commitment): Path<B256>,
) -> Response {
    let Some(root) =
        DbExt::<Keccak256>::get_root_for_leaf(&*state.kv_db, commitment).expect("DB error")
    else {
        return not_found();
    };

    let id = match sql::get_attestation_result(&state.sql_pool, root).await {
        Ok((_, AttestationResult::Pending)) => return not_found(),
        Ok((_, AttestationResult::MaxAttemptsExceeded)) => return internal_server_error(),
        Ok((id, AttestationResult::Success)) => id,
        Err(e) => {
            error!("SQL error: {e}");
            return internal_server_error();
        }
    };

    let Ok(Some(attestation)) = sql::get_last_successful_attest_attempt(&state.sql_pool, id).await
    else {
        error!("get_last_successful_attest_attempt failed for attestation_id {id} unexpectedly");
        return internal_server_error();
    };

    let trie = DbExt::<Keccak256>::load_trie(&*state.kv_db, root)
        .expect("DB error")
        .expect("bug: entry not found");

    let commitment = Output::<Keccak256>::from_slice(commitment.as_slice());
    let proof = trie
        .get_proof_iter(commitment)
        .expect("bug: proof not found");

    let mut builder = Timestamp::builder();
    builder.merkle_proof(proof);

    let timestamp = builder
        .attest(EASTimestamped {
            chain: Chain::from_id(attestation.chain_id),
        })
        .unwrap();

    let mut buf = BytesMut::with_capacity(128);
    timestamp.encode(&mut buf).unwrap();

    #[cfg(any(debug_assertions, not(feature = "performance")))]
    trace!(encoded_length = buf.len(), timestamp = ?timestamp);

    let mut headers = HeaderMap::with_capacity(2);
    headers.insert(header::CACHE_CONTROL, PUBLIC_IMMUTABLE.clone());
    headers.insert(header::CONTENT_TYPE, CONTENT_TYPE_OCTET_STREAM.clone());
    buf.freeze().into_response()
}
