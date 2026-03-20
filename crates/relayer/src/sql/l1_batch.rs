use alloy_primitives::{B256, ChainId};
use eyre::{Context, ContextCompat};
use jiff::Timestamp;
use serde::Serialize;
use sqlx::{Executor, Sqlite, SqliteConnection};
use uts_contracts::manager::events::{L1BatchArrived, L1BatchFinalized};
use uts_sql::{TextWrapper, define_text_enum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::IntoStaticStr, strum::EnumString, Serialize)]
pub enum L1BatchStatus {
    Collected,
    L1Sent,
    L1Mined,
    L2Received,
    L2FinalizeTxSent,
    L2Finalized,
}
define_text_enum!(L1BatchStatus);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub struct L1Batch {
    pub id: i64,
    pub l2_chain_id: ChainId,
    pub start_index: i64,
    pub count: i64,
    pub root: B256,
    pub l1_tx_hash: Option<B256>,
    pub l2_tx_hash: Option<B256>,
    pub status: L1BatchStatus,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

pub async fn count_l1batches<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
) -> eyre::Result<i64> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM l1_batch
        "#
    )
    .fetch_one(executor)
    .await
    .context("count l1 batches")?;
    Ok(count)
}

/// Insert a new L1 batch or update the existing one with the same (l2_chain_id, start_index) combination.
#[instrument(skip(executor), ret(level = "trace"), err)]
pub async fn upsert_l1batch<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    l2_chain_id: ChainId,
    root: B256,
    start_index: i64,
    count: i64,
    status: L1BatchStatus,
) -> eyre::Result<Option<L1Batch>> {
    let l2_chain_id: i64 = l2_chain_id.try_into().context("i64 overflow")?;
    let root = TextWrapper(root);

    let rec = sqlx::query!(
        r#"
        INSERT INTO l1_batch (l2_chain_id, start_index, count, root, status)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(l2_chain_id, start_index) DO UPDATE SET
            count = excluded.count,
            root = excluded.root,
            status = excluded.status
        WHERE
            CASE excluded.status
                WHEN 'Collected' THEN 1
                WHEN 'L1Sent' THEN 2
                WHEN 'L1Mined' THEN 3
                WHEN 'L2Received' THEN 4
                WHEN 'L2FinalizeTxSent' THEN 5
                WHEN 'L2Finalized' THEN 6
                ELSE 0
            END
            >=
            CASE l1_batch.status
                WHEN 'Collected' THEN 1
                WHEN 'L1Sent' THEN 2
                WHEN 'L1Mined' THEN 3
                WHEN 'L2Received' THEN 4
                WHEN 'L2FinalizeTxSent' THEN 5
                WHEN 'L2Finalized' THEN 6
                ELSE 0
            END
        RETURNING
            id,
            l1_tx_hash as "l1_tx_hash: TextWrapper<B256>",
            l2_tx_hash as "l2_tx_hash: TextWrapper<B256>",
            created_at,
            updated_at
        "#,
        l2_chain_id,
        start_index,
        count,
        root,
        status
    )
    .fetch_optional(executor)
    .await
    .context("upsert l1batch")?;

    let Some(rec) = rec else {
        return Ok(None);
    };

    Ok(Some(L1Batch {
        id: rec.id,
        l2_chain_id: l2_chain_id as u64,
        start_index,
        count,
        root: root.0,
        status,
        l1_tx_hash: rec.l1_tx_hash.map(|h| h.0),
        l2_tx_hash: rec.l2_tx_hash.map(|h| h.0),
        created_at: Timestamp::from_second(rec.created_at).context("timestamp overflow")?,
        updated_at: Timestamp::from_second(rec.updated_at).context("timestamp overflow")?,
    }))
}

#[instrument(skip(executor), err)]
pub async fn update_l1batch_to_l1_sent<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    batch_id: i64,
    l1_tx_hash: B256,
) -> eyre::Result<()> {
    let l1_tx_hash = TextWrapper(l1_tx_hash);
    sqlx::query!(
        r#"
        UPDATE l1_batch
        SET l1_tx_hash = ?1, status = ?2, updated_at = unixepoch()
        WHERE id = ?3
        "#,
        l1_tx_hash,
        L1BatchStatus::L1Sent,
        batch_id
    )
    .execute(executor)
    .await
    .context("update l1batch to l1 sent")?;
    Ok(())
}

#[instrument(skip(executor), err)]
pub async fn update_l1batch_to_l2finalize_tx_sent<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    batch_id: i64,
    l2_tx_hash: B256,
) -> eyre::Result<()> {
    let l2_tx_hash = TextWrapper(l2_tx_hash);
    sqlx::query!(
        r#"
        UPDATE l1_batch
        SET l2_tx_hash = ?1, status = ?2, updated_at = unixepoch()
        WHERE id = ?3
        "#,
        l2_tx_hash,
        L1BatchStatus::L2FinalizeTxSent,
        batch_id
    )
    .execute(executor)
    .await
    .context("update l1batch to l2 finalize tx sent")?;
    Ok(())
}

#[instrument(skip(executor), err)]
pub async fn compare_and_set_l1batch_status<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    batch_id: i64,
    current_status: L1BatchStatus,
    new_status: L1BatchStatus,
) -> eyre::Result<()> {
    sqlx::query!(
        r#"
        UPDATE l1_batch
        SET status = ?1, updated_at = unixepoch()
        WHERE id = ?2 AND status = ?3
        "#,
        new_status,
        batch_id,
        current_status
    )
    .execute(executor)
    .await
    .context("compare and set l1batch status")?;
    Ok(())
}

#[instrument(skip(executor), ret(level = "trace"), err)]
pub async fn get_latest_l1batch<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
) -> eyre::Result<Option<L1Batch>> {
    let rec = sqlx::query!(
        r#"
        SELECT
            id,
            l2_chain_id,
            start_index,
            count,
            root as "root: TextWrapper<B256>",
            l1_tx_hash as "l1_tx_hash: TextWrapper<B256>",
            l2_tx_hash as "l2_tx_hash: TextWrapper<B256>",
            status as "status: L1BatchStatus",
            created_at,
            updated_at
        FROM l1_batch
        ORDER BY id DESC LIMIT 1
        "#
    )
    .fetch_optional(executor)
    .await
    .context("select latest l1batch")?;

    let Some(rec) = rec else {
        return Ok(None);
    };
    Ok(Some(L1Batch {
        id: rec.id,
        l2_chain_id: rec.l2_chain_id as u64,
        start_index: rec.start_index,
        count: rec.count,
        root: rec.root.0,
        status: rec.status,
        l1_tx_hash: rec.l1_tx_hash.map(|h| h.0),
        l2_tx_hash: rec.l2_tx_hash.map(|h| h.0),
        created_at: Timestamp::from_second(rec.created_at).context("timestamp overflow")?,
        updated_at: Timestamp::from_second(rec.updated_at).context("timestamp overflow")?,
    }))
}

#[instrument(skip(executor), err)]
pub async fn insert_l1batch_arrived(
    executor: &'_ mut SqliteConnection,
    chain_id: ChainId,
    log_id: i64,
    event: L1BatchArrived,
) -> eyre::Result<()> {
    let Some(l1_batch) = upsert_l1batch(
        &mut *executor,
        chain_id,
        event.claimedRoot,
        event.startIndex.to(),
        event.count.to(),
        L1BatchStatus::L2Received,
    )
    .await?
    else {
        warn!("Existing L1 batch has newer status, skipping insert_l1batch_arrived");
        return Ok(());
    };

    let l1_block_attested: i64 = event.l1BlockAttested.try_into().context("i64 overflow")?;
    let l1_timestamp_attested: i64 = event
        .l1TimestampAttested
        .try_into()
        .context("i64 overflow")?;
    let l2_block_number: i64 = event.l2BlockNumber.try_into().context("i64 overflow")?;
    let l2_timestamp_received: i64 = event
        .l2TimestampReceived
        .try_into()
        .context("i64 overflow")?;

    sqlx::query!(
        r#"
        INSERT INTO l1_batch_arrived (l1_batch_id, internal_log_id, l1_block_attested, l1_timestamp_attested, l2_block_number, l2_timestamp_received)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT DO NOTHING
        "#,
        l1_batch.id,
        log_id,
        l1_block_attested,
        l1_timestamp_attested,
        l2_block_number,
        l2_timestamp_received,
    ).execute(executor).await.context("insert l1 batch arrived")?;

    Ok(())
}

#[instrument(skip(executor), err)]
pub async fn insert_l1batch_finalized(
    executor: &'_ mut SqliteConnection,
    chain_id: ChainId,
    log_id: i64,
    event: L1BatchFinalized,
) -> eyre::Result<()> {
    let l1_batch = upsert_l1batch(
        &mut *executor,
        chain_id,
        event.merkleRoot,
        event.startIndex.to(),
        event.count.to(),
        L1BatchStatus::L2Finalized,
    )
    .await?
    .context("finalized is the final status, but upsert did not return the batch")?;

    let l2_block_number: i64 = event.l2BlockNumber.try_into().context("i64 overflow")?;
    let l2_timestamp_finalized: i64 = event
        .l2TimestampFinalized
        .try_into()
        .context("i64 overflow")?;

    sqlx::query!(
        r#"
        INSERT INTO l1_batch_finalized (l1_batch_id, internal_log_id, l2_block_number, l2_timestamp_finalized)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT DO NOTHING
        "#,
        l1_batch.id,
        log_id,
        l2_block_number,
        l2_timestamp_finalized,
    ).execute(executor).await.context("insert l1 batch finalized")?;

    Ok(())
}
