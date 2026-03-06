use crate::MAX_RETRIES;
use alloy_primitives::{B256, BlockNumber, ChainId, TxHash, private::serde::Serialize};
use sqlx::SqlitePool;
use types::Wrapper;

mod types;

/// A struct representing a pending attestation that has been created but not yet attested on-chain.
#[derive(Debug, Copy, Clone)]
pub struct PendingAttestation {
    /// The ID of the attestation in the database.
    pub id: i64,
    /// The merkle root that is being attested.
    pub trie_root: B256,
    /// The timestamp when the attestation was created (Unix timestamp).
    pub created_at: u64,
    /// The timestamp when the attestation was last updated (Unix timestamp).
    pub updated_at: u64,
    /// The current result of the attestation attempt (pending, success, or max attempts exceeded).
    pub result: AttestationResult,
}

/// An enum representing the result of an attestation attempt.
#[derive(Debug, Copy, Clone, strum::IntoStaticStr, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AttestationResult {
    /// The attestation is still pending and has not yet been attested on-chain.
    Pending,
    /// The attestation was successfully attested on-chain.
    Success,
    /// Exceeded maximum number of attempts to attest on-chain.
    MaxAttemptsExceeded,
}

/// A struct representing an attempt to attest a merkle root on-chain
#[derive(Debug, Copy, Clone)]
pub struct AttestAttempt {
    /// The ID of the attestation this attempt is for.
    pub attestation_id: i64,
    /// The chain ID on which the attest attempt was made.
    pub chain_id: ChainId,
    /// The transaction details if the attempt was successful, None if the attempt failed (e.g. due to revert or RPC error).
    pub tx: Option<AttestTx>,
}

/// A struct representing a successful attestation transaction on-chain.
#[derive(Debug, Copy, Clone)]
pub struct AttestTx {
    /// The hash of the transaction that attested the merkle root on-chain.
    pub tx_hash: TxHash,
    /// The block number in which the transaction was included.
    pub block_number: BlockNumber,
}

/// A struct representing statistics about pending attestations in the database.
#[derive(Debug, Copy, Clone, Serialize)]
pub struct Stats {
    /// The total number of pending attestations in the database.
    pub pending: usize,
    /// The total number of successful attempts for pending attestations.
    pub success: usize,
    /// The total number of failed attempts for pending attestations.
    pub failed: usize,
    /// The unix timestamp of the last attestation attempt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_attempt: Option<u64>,
    /// The details of the last successful attestation attempt, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_success: Option<SuccessAttempt>,
}

/// A struct representing the details of a successful attestation attempt.
#[derive(Debug, Copy, Clone, Serialize)]
pub struct SuccessAttempt {
    /// The chain ID on which the successful attest attempt was made.
    pub chain_id: ChainId,
    /// The hash of the transaction that successfully attested the merkle root on-chain.
    pub tx_hash: TxHash,
    /// The block number in which the successful transaction was included.
    pub block_number: BlockNumber,
}

/// Create a new pending attestation in the database and return its ID.
pub async fn new_pending_attestation(pool: &SqlitePool, root: B256) -> sqlx::Result<i64> {
    let root = Wrapper(root);

    let rec = sqlx::query!(
        r#"INSERT INTO pending_attestations (trie_root) VALUES (?) RETURNING id"#,
        root,
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.id)
}

/// Load all pending attestations from the database.
pub async fn load_all_pending_attestations(
    pool: &SqlitePool,
) -> sqlx::Result<Vec<PendingAttestation>> {
    let recs = sqlx::query!(
        r#"
        SELECT
            id as "id!",
            trie_root as "trie_root: Wrapper<B256>",
            created_at as "created_at!",
            updated_at as "updated_at!"
        FROM pending_attestations
        WHERE result = 'pending'
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(recs
        .into_iter()
        .map(|rec| PendingAttestation {
            id: rec.id,
            trie_root: rec.trie_root.0,
            created_at: rec.created_at as u64,
            updated_at: rec.updated_at as u64,
            // we only load pending attestations here
            result: AttestationResult::Pending,
        })
        .collect())
}

/// Get the attestation result for a given trie root. Returns the attestation ID and result if found.
pub async fn get_attestation_result(
    pool: &SqlitePool,
    trie_root: B256,
) -> sqlx::Result<(i64, AttestationResult)> {
    let trie_root = Wrapper(trie_root);

    let rec = sqlx::query!(
        r#"
        SELECT id, result as "result: AttestationResult"
        FROM pending_attestations
        WHERE trie_root = ?1
        "#,
        trie_root,
    )
    .fetch_one(pool)
    .await?;

    Ok((rec.id, rec.result))
}

/// Get the last successful attest attempt for a given attestation ID, if any.
pub async fn get_last_successful_attest_attempt(
    pool: &SqlitePool,
    attestation_id: i64,
) -> sqlx::Result<Option<AttestAttempt>> {
    let rec = sqlx::query!(
        r#"
        SELECT
            attestation_id,
            chain_id,
            tx_hash as "tx_hash: Wrapper<TxHash>",
            block_number as "block_number!"
        FROM attest_attempts
        WHERE attestation_id = ?1 AND tx_hash IS NOT NULL AND block_number IS NOT NULL
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        attestation_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(rec.map(|r| AttestAttempt {
        attestation_id: r.attestation_id,
        chain_id: ChainId::from(r.chain_id as u64),
        tx: r.tx_hash.map(|tx_hash| AttestTx {
            tx_hash: tx_hash.0,
            block_number: r.block_number as u64,
        }),
    }))
}

/// Create a new attest attempt for a given attestation ID and chain ID.
///
/// If the attempt was successful, also update the pending attestation result to success.
/// If the attempt failed, check if we have exceeded the maximum number of attempts and update the
/// result to max_attempts_exceeded if so.
pub async fn new_attest_attempt(
    pool: &SqlitePool,
    attestation_id: i64,
    chain_id: ChainId,
    may_success: Option<AttestTx>,
) -> sqlx::Result<()> {
    let chain_id = chain_id as i64;
    let tx_hash = may_success.map(|tx| Wrapper(tx.tx_hash));
    let block_number = may_success.map(|tx| tx.block_number as i64);

    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        INSERT INTO attest_attempts (attestation_id, chain_id, tx_hash, block_number)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        attestation_id,
        chain_id,
        tx_hash,
        block_number,
    )
    .execute(&mut *tx)
    .await?;

    if may_success.is_some() {
        sqlx::query!(
            r#"
            UPDATE pending_attestations
            SET result = 'success', updated_at = unixepoch()
            WHERE id = ?1
            "#,
            attestation_id,
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // check if we have exceeded max attempts
        let total_attempts = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM attest_attempts
            WHERE attestation_id = ?1
            "#,
            attestation_id,
        )
        .fetch_one(&mut *tx)
        .await?;
        if total_attempts >= MAX_RETRIES as i64 {
            sqlx::query!(
                r#"
                UPDATE pending_attestations
                SET result = 'max_attempts_exceeded', updated_at = unixepoch()
                WHERE id = ?1
                "#,
                attestation_id,
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    Ok(())
}

/// Get stats
pub async fn get_stats(pool: &SqlitePool) -> sqlx::Result<Stats> {
    let mut tx = pool.begin().await?;

    let counts = sqlx::query!(
        r#"
        SELECT
            SUM(CASE WHEN result = 'pending' THEN 1 ELSE 0 END) as "pending!",
            SUM(CASE WHEN result = 'success' THEN 1 ELSE 0 END) as "success!",
            SUM(CASE WHEN result <> 'pending' AND result <> 'success' THEN 1 ELSE 0 END) as "failed!"
        FROM pending_attestations
        "#
    )
        .fetch_one(&mut *tx)
        .await?;

    let last_attempt: Option<i64> = sqlx::query_scalar!(
        r#"SELECT created_at FROM attest_attempts ORDER BY created_at DESC LIMIT 1"#
    )
    .fetch_optional(&mut *tx)
    .await?;

    let last_success_rec = sqlx::query!(
        r#"
        SELECT
            chain_id,
            tx_hash as "tx_hash!: Wrapper<TxHash>",
            block_number as "block_number!"
        FROM attest_attempts
        WHERE tx_hash IS NOT NULL AND block_number IS NOT NULL
        ORDER BY created_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(&mut *tx)
    .await?;

    let last_success = last_success_rec.map(|rec| SuccessAttempt {
        chain_id: rec.chain_id as u64,
        tx_hash: rec.tx_hash.0,
        block_number: rec.block_number as u64,
    });

    tx.commit().await?;

    Ok(Stats {
        pending: counts.pending as usize,
        success: counts.success as usize,
        failed: counts.failed as usize,
        last_attempt: last_attempt.map(|ts| ts as u64),
        last_success,
    })
}
