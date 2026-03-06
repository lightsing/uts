use alloy_primitives::{B256, ChainId};
use eyre::Context;
use jiff::Timestamp;
use sqlx::{Executor, Sqlite, SqliteConnection};
use uts_contracts::manager::events::L1BatchArrived;
use uts_sql::{TextWrapper, define_text_enum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::IntoStaticStr, strum::EnumString)]
pub enum L1BatchStatus {
    L1Sent,
    L2Received,
    L2Finalizing,
    L2Finalized,
}
define_text_enum!(L1BatchStatus);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct L1Batch {
    pub id: i64,
    pub l2_chain_id: ChainId,
    pub start_index: i64,
    pub count: i64,
    pub root: B256,
    pub status: L1BatchStatus,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Insert a new L1 batch or update the existing one with the same (l2_chain_id, start_index, count) combination.
pub async fn upsert_l1batch<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    l2_chain_id: ChainId,
    root: B256,
    start_index: i64,
    count: i64,
    status: L1BatchStatus,
) -> eyre::Result<L1Batch> {
    let l2_chain_id: i64 = l2_chain_id.try_into().context("i64 overflow")?;
    let root = TextWrapper(root);

    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO l1_batch (l2_chain_id, start_index, count, root, status)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(l2_chain_id, start_index, count) DO UPDATE SET
            root = excluded.root,
            status = excluded.status
        RETURNING id
        "#,
        l2_chain_id,
        start_index,
        count,
        root,
        status
    )
    .fetch_one(executor)
    .await?;
    Ok(L1Batch {
        id,
        l2_chain_id: l2_chain_id as u64,
        start_index,
        count,
        root: root.0,
        status,
        created_at: Timestamp::now(),
        updated_at: Timestamp::now(),
    })
}

pub async fn update_l1batch_status<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    batch_id: i64,
    new_status: L1BatchStatus,
) -> eyre::Result<()> {
    sqlx::query!(
        r#"
        UPDATE l1_batch
        SET status = ?1, updated_at = unixepoch()
        WHERE id = ?2
        "#,
        new_status,
        batch_id
    )
    .execute(executor)
    .await?;
    Ok(())
}

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
            status as "status: L1BatchStatus",
            created_at,
            updated_at
        FROM l1_batch
        ORDER BY id DESC LIMIT 1
        "#
    )
    .fetch_optional(executor)
    .await?;

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
        created_at: Timestamp::from_second(rec.created_at)?,
        updated_at: Timestamp::from_second(rec.updated_at)?,
    }))
}

pub async fn insert_l1batch_arrived(
    executor: &'_ mut SqliteConnection,
    chain_id: ChainId,
    log_id: i64,
    event: L1BatchArrived,
) -> eyre::Result<()> {
    let l1_batch = upsert_l1batch(
        &mut *executor,
        chain_id,
        event.claimedRoot,
        event.startIndex.to(),
        event.count.to(),
        L1BatchStatus::L2Received,
    )
    .await?;

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
        "#,
        l1_batch.id,
        log_id,
        l1_block_attested,
        l1_timestamp_attested,
        l2_block_number,
        l2_timestamp_received,
    ).execute(executor).await?;

    Ok(())
}

pub async fn insert_l1batch_finalized(
    executor: &'_ mut SqliteConnection,
    chain_id: ChainId,
    log_id: i64,
    event: L1BatchArrived,
) -> eyre::Result<()> {
    let l1_batch = upsert_l1batch(
        &mut *executor,
        chain_id,
        event.claimedRoot,
        event.startIndex.to(),
        event.count.to(),
        L1BatchStatus::L2Finalized,
    )
    .await?;

    let l2_block_number: i64 = event.l2BlockNumber.try_into().context("i64 overflow")?;
    let l2_timestamp_finalized: i64 = event
        .l2TimestampReceived
        .try_into()
        .context("i64 overflow")?;

    sqlx::query!(
        r#"
        INSERT INTO l1_batch_finalized (l1_batch_id, internal_log_id, l2_block_number, l2_timestamp_finalized)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        l1_batch.id,
        log_id,
        l2_block_number,
        l2_timestamp_finalized,
    ).execute(executor).await?;

    Ok(())
}
