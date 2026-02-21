use crate::{AppState, time::current_time_sec};
use alloy_primitives::{B256, hex};
use alloy_signer::SignerSync;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bump_scope::Bump;
use bytes::BytesMut;
use digest::Digest;
use sha3::Keccak256;
use std::{cell::RefCell, sync::Arc};
use uts_bmt::{FlatMerkleTree, NodePosition};
use uts_core::{
    codec::{
        Encode,
        v1::{EthereumUTSAttestation, PendingAttestation, Timestamp},
    },
    utils::Hexed,
};
use uts_stamper::DbExt;

/// Maximum digest size accepted by the endpoint.
pub const MAX_DIGEST_SIZE: usize = 64; // e.g., SHA3-512

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
/// Submit digest to calendar server and get pending timestamp in response.
pub async fn submit_digest(State(state): State<Arc<AppState>>, digest: Bytes) -> Response {
    let (output, commitment) = submit_digest_inner(digest, &state.signer);
    match state.journal.try_commit(&commitment) {
        Err(_) => {
            return (StatusCode::SERVICE_UNAVAILABLE, r#"{"err":"server busy"}"#).into_response();
        } // journal is full
        Ok(fut) => fut.await,
    }
    output.into_response()
}

// TODO: We need to benchmark this.
/// inner function to submit digest, returns (encoded timestamp, commitment)
pub fn submit_digest_inner(digest: Bytes, signer: impl SignerSync) -> (Bytes, [u8; 32]) {
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

        let undeniable_sig = signer.sign_hash_sync(&hash.0.into()).unwrap();
        undeniable_sig.as_erc2098()
    };

    #[cfg(any(debug_assertions, not(feature = "performance")))]
    trace!(
        recv_timestamp = ?Hexed(&recv_timestamp),
        digest = ?Hexed(&digest),
        undeniable_sig = ?Hexed(&undeniable_sig),
    );

    BUMP.with(|bump| {
        let mut bump = bump.borrow_mut();
        bump.reset();

        let builder = Timestamp::builder_in(&*bump)
            .prepend(recv_timestamp.to_vec_in(&bump))
            .append(undeniable_sig.to_vec_in(&bump))
            .keccak256();

        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(&builder.commitment(&digest));

        let timestamp = builder
            .attest(PendingAttestation {
                uri: "http://localhost:3000".into(),
            })
            .unwrap();

        // copy data out of bump
        // TODO: eliminate this allocation by reusing from a pool
        // TODO: wrap the buffer with a drop trait to return to pool
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
    const PRE_ALLOCATION_SIZE_HINT: usize = 4096;
    thread_local! {
        // We don't have `.await` in this function, so it's safe to borrow thread local.
        static BUMP: RefCell<Bump> = RefCell::new(Bump::with_size(PRE_ALLOCATION_SIZE_HINT));
    }

    let Some(root) = state.db.get_root_for_leaf(commitment).expect("DB error") else {
        return (StatusCode::NOT_FOUND, r#"{"err":"timestamp not found"}"#).into_response();
    };
    let entry = state
        .db
        .load_entry(root)
        .expect("DB error")
        .expect("bug: entry not found");
    let trie: FlatMerkleTree<Keccak256> = entry.trie();

    let mut proof_iter = trie
        .get_proof_iter(bytemuck::cast_ref(&*commitment))
        .expect("bug: proof not found");
    let output = BUMP.with(|bump| {
        let mut bump = bump.borrow_mut();
        bump.reset();

        let mut builder = Timestamp::builder_in(&*bump);

        while let Some((side, sibling_hash)) = proof_iter.next() {
            builder = match side {
                NodePosition::Left => builder.append(sibling_hash.to_vec_in(&bump)),
                NodePosition::Right => builder.prepend(sibling_hash.to_vec_in(&bump)),
            }
            .keccak256();
        }
        let timestamp = builder
            .attest(EthereumUTSAttestation::new(
                entry.chain_id,
                entry.height,
                Default::default(),
            ))
            .unwrap();

        // copy data out of bump
        // TODO: eliminate this allocation by reusing from a pool
        // TODO: wrap the buffer with a drop trait to return to pool
        let mut buf = BytesMut::with_capacity(128);
        timestamp.encode(&mut buf).unwrap();

        #[cfg(any(debug_assertions, not(feature = "performance")))]
        trace!(encoded_length = buf.len(), timestamp = ?timestamp);

        buf.freeze()
    });

    output.into_response()
}
