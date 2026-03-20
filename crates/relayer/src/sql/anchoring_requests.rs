use alloy_primitives::B256;
use eyre::Context;
use sqlx::{Executor, Sqlite};
use uts_contracts::manager::events::L1AnchoringQueued;
use uts_sql::TextWrapper;

#[instrument(skip(executor), ret(level = "trace"), err)]
pub async fn insert_l1anchoring_queued<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    log_id: i64,
    event: L1AnchoringQueued,
) -> eyre::Result<()> {
    let attestation_id = TextWrapper(event.attestationId);
    let root = TextWrapper(event.root);
    let queue_index: i64 = event.queueIndex.to();
    let fee: i64 = event.fee.to();
    let block_number: i64 = event.blockNumber.to();
    let timestamp: i64 = event.timestamp.to();

    sqlx::query!(
        r#"
        INSERT INTO l1_anchoring_queued (internal_log_id, attestation_id, root, queue_index, fee, block_number, timestamp)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT DO NOTHING
        "#,
        log_id,
        attestation_id,
        root,
        queue_index,
        fee,
        block_number,
        timestamp,
    )
        .execute(executor)
        .await.context("insert l1anchoring queued")?;
    Ok(())
}

#[instrument(skip(executor), ret(level = "trace"), err)]
pub async fn count_pending_events<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    next_start_index: i64,
) -> eyre::Result<i64> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM l1_anchoring_queued
        WHERE queue_index >= ?1
        "#,
        next_start_index,
    )
    .fetch_one(executor)
    .await
    .context("count pending events")?;
    Ok(count)
}

#[instrument(skip(executor), ret(level = "trace"), err)]
pub async fn load_roots_in_range<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    start_index: i64,
    count: i64,
) -> eyre::Result<Vec<B256>> {
    let rows = sqlx::query_scalar!(
        r#"
        SELECT root as "root: TextWrapper<B256>"
        FROM l1_anchoring_queued
        WHERE queue_index >= ?1
        ORDER BY queue_index
        LIMIT ?2
        "#,
        start_index,
        count,
    )
    .fetch_all(executor)
    .await
    .context("load roots in range")?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}
