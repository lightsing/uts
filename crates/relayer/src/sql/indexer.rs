use alloy_primitives::{B256, BlockNumber, ChainId};
use eyre::Context;
use sqlx::{Executor, Sqlite};
use uts_sql::TextWrapper;

// CREATE TABLE IF NOT EXISTS indexer_cursors (
//   chain_id INTEGER PRIMARY KEY,
//   event_signature TEXT NOT NULL,
//   last_indexed_block INTEGER NOT NULL,
//   updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
// );
#[instrument(skip(executor), err)]
pub async fn update_cursor<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    chain_id: ChainId,
    event_signature_hash: B256,
    last_indexed_block: BlockNumber,
) -> eyre::Result<()> {
    let chain_id: i64 = chain_id.try_into().context("i64 overflow")?;
    let event_signature_hash = TextWrapper(event_signature_hash);
    let last_indexed_block: i64 = last_indexed_block.try_into().context("i64 overflow")?;
    sqlx::query!(
        r#"
        INSERT INTO indexer_cursors (chain_id, event_signature_hash, last_indexed_block)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(chain_id, event_signature_hash) DO UPDATE SET last_indexed_block = excluded.last_indexed_block, updated_at = CURRENT_TIMESTAMP
        "#,
        chain_id,
        event_signature_hash,
        last_indexed_block,
    )
        .execute(executor)
        .await.context("update cursor")?;
    Ok(())
}

#[instrument(skip(executor), ret(level = "trace"), err)]
pub async fn load_cursor<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    chain_id: ChainId,
    event_signature_hash: B256,
) -> eyre::Result<Option<BlockNumber>> {
    let chain_id: i64 = chain_id.try_into().context("i64 overflow")?;
    let event_signature_hash = TextWrapper(event_signature_hash);
    let record = sqlx::query!(
        r#"
        SELECT last_indexed_block FROM indexer_cursors
        WHERE chain_id = ?1 AND event_signature_hash = ?2
        "#,
        chain_id,
        event_signature_hash,
    )
    .fetch_optional(executor)
    .await
    .context("load cursor")?;

    Ok(record.map(|r| r.last_indexed_block as BlockNumber))
}
