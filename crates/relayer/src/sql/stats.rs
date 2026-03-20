use alloy_primitives::U256;
use eyre::Context;
use serde::Serialize;
use sqlx::{Executor, Sqlite};
use uts_sql::TextWrapper;

// CREATE TABLE IF NOT EXISTS batch_fee (
//   id INTEGER PRIMARY KEY AUTOINCREMENT,
//   internal_batch_id INTEGER NOT NULL REFERENCES l1_batch (id) ON DELETE CASCADE,
//
//   l1_gas_fee TEXT NOT NULL DEFAULT 0,
//   l2_gas_fee TEXT NOT NULL DEFAULT 0,
//   cross_chain_fee TEXT NOT NULL DEFAULT 0,
// );
// CREATE UNIQUE INDEX IF NOT EXISTS idx_batch_fee_batch_id ON batch_fee (internal_batch_id);
pub async fn set_l1_gas_fee<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    batch_id: i64,
    l1_gas_fee: U256,
) -> eyre::Result<()> {
    let l1_gas_fee = TextWrapper(l1_gas_fee);
    sqlx::query!(
        r#"
        INSERT INTO batch_fee (internal_batch_id, l1_gas_fee, l2_gas_fee, cross_chain_fee)
        VALUES (?1, ?2, 0, 0)
        ON CONFLICT(internal_batch_id) DO UPDATE SET l1_gas_fee = excluded.l1_gas_fee
        "#,
        batch_id,
        l1_gas_fee,
    )
    .execute(executor)
    .await
    .context("insert or update batch fee")?;

    Ok(())
}

pub async fn set_l2_gas_fee<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    batch_id: i64,
    l2_gas_fee: U256,
) -> eyre::Result<()> {
    let l2_gas_fee = TextWrapper(l2_gas_fee);
    sqlx::query!(
        r#"
        INSERT INTO batch_fee (internal_batch_id, l1_gas_fee, l2_gas_fee, cross_chain_fee)
        VALUES (?1, 0, ?2, 0)
        ON CONFLICT(internal_batch_id) DO UPDATE SET l2_gas_fee = excluded.l2_gas_fee
        "#,
        batch_id,
        l2_gas_fee,
    )
    .execute(executor)
    .await
    .context("insert or update batch fee")?;

    Ok(())
}

pub async fn set_cross_chain_fee<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
    batch_id: i64,
    cross_chain_fee: U256,
) -> eyre::Result<()> {
    let cross_chain_fee = TextWrapper(cross_chain_fee);
    sqlx::query!(
        r#"
        INSERT INTO batch_fee (internal_batch_id, l1_gas_fee, l2_gas_fee, cross_chain_fee)
        VALUES (?1, 0, 0, ?2)
        ON CONFLICT(internal_batch_id) DO UPDATE SET cross_chain_fee = excluded.cross_chain_fee
        "#,
        batch_id,
        cross_chain_fee,
    )
    .execute(executor)
    .await
    .context("insert or update batch fee")?;
    Ok(())
}

#[derive(Debug, Default, Serialize)]
pub struct CulminationCosts {
    pub l1_gas_fee: f32,
    pub l2_gas_fee: f32,
    pub cross_chain_fee: f32,
}

pub async fn get_culmination_costs<'e, E: Executor<'e, Database = Sqlite>>(
    executor: E,
) -> eyre::Result<CulminationCosts> {
    let record = sqlx::query!(
        r#"
        SELECT SUM(CAST(l1_gas_fee AS REAL)) as l1_gas_fee,
               SUM(CAST(l2_gas_fee AS REAL)) as l2_gas_fee,
               SUM(CAST(cross_chain_fee AS REAL)) as cross_chain_fee
        FROM batch_fee
        "#,
    )
    .fetch_one(executor)
    .await
    .context("get culmination costs")?;

    Ok(CulminationCosts {
        l1_gas_fee: record.l1_gas_fee.unwrap_or(0.0) as f32,
        l2_gas_fee: record.l2_gas_fee.unwrap_or(0.0) as f32,
        cross_chain_fee: record.cross_chain_fee.unwrap_or(0.0) as f32,
    })
}
