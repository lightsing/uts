use alloy_primitives::{B256, BlockNumber, ChainId};
use alloy_rpc_types_eth::Log;
use eyre::{Context, ContextCompat};
use sqlx::{Executor, Sqlite};
use uts_sql::TextWrapper;

// CREATE TABLE IF NOT EXISTS indexer_cursors (
//   chain_id INTEGER PRIMARY KEY,
//   event_signature TEXT NOT NULL,
//   last_indexed_block INTEGER NOT NULL,
//   updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
// );
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
        .await?;
    Ok(())
}

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
    .await?;

    Ok(record.map(|r| r.last_indexed_block as BlockNumber))
}

// CREATE TABLE IF NOT EXISTS eth_block (
//    id INTEGER PRIMARY KEY AUTOINCREMENT,
//    chain_id INTEGER NOT NULL,
//    block_hash TEXT NOT NULL,
//    block_number INTEGER NOT NULL,
//    block_timestamp INTEGER NOT NULL
// );
//
// CREATE TABLE IF NOT EXISTS eth_transaction (
//   id               INTEGER PRIMARY KEY AUTOINCREMENT,
//   internal_block_id     INTEGER NOT NULL REFERENCES eth_block (id) ON DELETE CASCADE,
//   transaction_index INTEGER NOT NULL,
//   transaction_hash  TEXT    NOT NULL
// );
//
// CREATE TABLE IF NOT EXISTS eth_log (
//   id              INTEGER PRIMARY KEY AUTOINCREMENT,
//   internal_transaction_id INTEGER NOT NULL REFERENCES eth_transaction (id) ON DELETE CASCADE,
//   log_index        INTEGER NOT NULL
// );
pub async fn insert_log<E>(executor: &mut E, chain_id: ChainId, log: Log) -> eyre::Result<i64>
where
    for<'e> &'e mut E: Executor<'e, Database = Sqlite>,
{
    let chain_id: i64 = chain_id.try_into().context("i64 overflow")?;
    let block_hash = TextWrapper(log.block_hash.context("missing block hash")?);
    let block_number: i64 = log
        .block_number
        .context("missing block number")?
        .try_into()
        .context("i64 overflow")?;
    let transaction_hash = TextWrapper(log.transaction_hash.context("missing transaction hash")?);
    let transaction_index: i64 = log
        .transaction_index
        .context("missing transaction index")?
        .try_into()
        .context("i64 overflow")?;
    let log_index: i64 = log
        .log_index
        .context("missing log index")?
        .try_into()
        .context("i64 overflow")?;

    let block_id = sqlx::query_scalar!(
        r#"
        INSERT INTO eth_block (chain_id, block_hash, block_number)
        VALUES (?1, ?2, ?3)
        ON CONFLICT DO NOTHING
        RETURNING id
        "#,
        chain_id,
        block_hash,
        block_number,
    )
    .fetch_one(&mut *executor)
    .await?;

    let transaction_id = sqlx::query_scalar!(
        r#"
        INSERT INTO eth_transaction (internal_block_id, transaction_index, transaction_hash)
        VALUES (?1, ?2, ?3)
        ON CONFLICT DO NOTHING
        RETURNING id as "id!"
        "#,
        block_id,
        transaction_index,
        transaction_hash,
    )
    .fetch_one(&mut *executor)
    .await?;

    let log_id = sqlx::query_scalar!(
        r#"
        INSERT INTO eth_log (internal_transaction_id, log_index)
        VALUES (?1, ?2)
        ON CONFLICT DO NOTHING
        RETURNING id as "id!"
        "#,
        transaction_id,
        log_index,
    )
    .fetch_one(&mut *executor)
    .await?;

    Ok(log_id)
}
