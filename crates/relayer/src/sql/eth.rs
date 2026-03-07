use alloy_primitives::{B256, BlockNumber, ChainId, U256};
use alloy_rpc_types_eth::{Log, TransactionReceipt};
use eyre::{Context, ContextCompat};
use sqlx::{Executor, Sqlite};
use uts_sql::TextWrapper;

/// Insert a new Ethereum block into the database. If a block with the same (chain_id, block_hash) already exists, do nothing.
#[instrument(skip(executor), ret(level = "trace"), err)]
pub async fn insert_block<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    chain_id: ChainId,
    block_hash: B256,
    block_number: BlockNumber,
) -> eyre::Result<i64> {
    let chain_id: i64 = chain_id.try_into().context("chain id overflow")?;
    let block_number: i64 = block_number.try_into().context("block number overflow")?;
    let block_hash = TextWrapper(block_hash);
    let block_id = sqlx::query_scalar!(
        r#"
        INSERT INTO eth_block (chain_id, block_hash, block_number)
        VALUES (?1, ?2, ?3)
        ON CONFLICT DO UPDATE SET block_number = excluded.block_number
        RETURNING id
        "#,
        chain_id,
        block_hash,
        block_number,
    )
    .fetch_one(executor)
    .await
    .context("insert block")?;
    Ok(block_id)
}

/// Insert a new Ethereum transaction into the database. If a transaction with the same (internal_block_id, transaction_index) already exists, do nothing.
#[instrument(skip(executor), ret(level = "trace"), err)]
pub async fn insert_transaction<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    block_id: i64,
    transaction_index: u64,
    transaction_hash: B256,
) -> eyre::Result<i64> {
    let transaction_index: i64 = transaction_index
        .try_into()
        .context("transaction index overflow")?;
    let transaction_hash = TextWrapper(transaction_hash);
    let transaction_id = sqlx::query_scalar!(
        r#"
        INSERT INTO eth_transaction (internal_block_id, transaction_index, transaction_hash)
        VALUES (?1, ?2, ?3)
        ON CONFLICT DO UPDATE SET transaction_hash = excluded.transaction_hash
        RETURNING id as "id!"
        "#,
        block_id,
        transaction_index,
        transaction_hash,
    )
    .fetch_one(executor)
    .await
    .context("insert transaction")?;
    Ok(transaction_id)
}

#[instrument(
    skip(executor, log),
    fields(
        %address = log.address(),
        %topic0 = log.topic0().unwrap_or_default(),
        %transaction_hash = log.transaction_hash.unwrap_or_default(),
        %log_index = log.log_index.unwrap_or_default(),
    ),
    ret(level = "trace"),
    err
)]
pub async fn insert_log<E>(executor: &mut E, chain_id: ChainId, log: &Log) -> eyre::Result<i64>
where
    for<'e> &'e mut E: Executor<'e, Database = Sqlite>,
{
    let block_hash = log.block_hash.context("missing block hash")?;
    let block_number = log.block_number.context("missing block number")?;
    let block_id = insert_block(&mut *executor, chain_id, block_hash, block_number).await?;

    let transaction_index = log.transaction_index.context("missing transaction index")?;
    let transaction_hash = log.transaction_hash.context("missing transaction hash")?;
    let transaction_id = insert_transaction(
        &mut *executor,
        block_id,
        transaction_index,
        transaction_hash,
    )
    .await?;

    let log_index: i64 = log
        .log_index
        .context("missing log index")?
        .try_into()
        .context("i64 overflow")?;

    let log_id = sqlx::query_scalar!(
        r#"
        INSERT INTO eth_log (internal_transaction_id, log_index)
        VALUES (?1, ?2)
        ON CONFLICT DO UPDATE SET log_index = excluded.log_index
        RETURNING id as "id!"
        "#,
        transaction_id,
        log_index,
    )
    .fetch_one(&mut *executor)
    .await
    .context("insert log")?;

    Ok(log_id)
}

#[instrument(skip(executor, tx_receipt), fields(%tx_hash = tx_receipt.transaction_hash), err)]
pub async fn insert_tx_receipt<E>(
    executor: &'_ mut E,
    chain_id: ChainId,
    tx_receipt: &TransactionReceipt,
) -> eyre::Result<()>
where
    for<'e> &'e mut E: Executor<'e, Database = Sqlite>,
{
    let block_hash = tx_receipt.block_hash.context("missing block hash")?;
    let block_number = tx_receipt.block_number.context("missing block number")?;
    let block_id = insert_block(&mut *executor, chain_id, block_hash, block_number).await?;

    let transaction_index = tx_receipt
        .transaction_index
        .context("missing transaction index")?;
    let transaction_id = insert_transaction(
        &mut *executor,
        block_id,
        transaction_index,
        tx_receipt.transaction_hash,
    )
    .await?;

    let from_address = TextWrapper(tx_receipt.from);
    let to_address = TextWrapper(tx_receipt.to.context("missing to address")?);
    let gas_used: i64 = tx_receipt.gas_used.try_into().context("i64 overflow")?;
    let effective_gas_price = TextWrapper(U256::from(tx_receipt.effective_gas_price));

    sqlx::query!(
        r#"
        INSERT INTO tx_receipt (internal_transaction_id, gas_used, effective_gas_price, from_address, to_address)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(internal_transaction_id) DO UPDATE SET
            gas_used = excluded.gas_used,
            effective_gas_price = excluded.effective_gas_price,
            from_address = excluded.from_address,
            to_address = excluded.to_address
        "#,
        transaction_id,
        gas_used,
        effective_gas_price,
        from_address,
        to_address,
    )
    .execute(&mut *executor)
    .await
        .context("insert tx receipt")?;

    Ok(())
}
