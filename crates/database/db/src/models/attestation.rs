//! Attestation model.

use alloy_primitives::{B256, TxHash};
use sea_orm::{ActiveValue, entity::prelude::*};
use serde::{Deserialize, Serialize};

/// Database model representing a UTS Attestation.
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "attestation")]
pub struct Model {
    /// Incremental index for entries into the table.
    #[sea_orm(primary_key)]
    pub index: i64,
    /// The merkle tree root to attest.
    #[sea_orm(unique)]
    pub root: Vec<u8>,
    /// The tx hash for the on-chain attestation.
    #[sea_orm(unique)]
    pub tx_hash: Vec<u8>,
    /// The block number in which the on-chain attestation was included.
    ///
    /// The "attestation" table is indexed by this column.
    #[sea_orm(indexed, nullable)]
    pub block_number: i64,
    /// The timestamp for the block in which the on-chain attestation was included.
    ///
    /// The "attestation" table is indexed by this column.
    #[sea_orm(indexed, nullable)]
    pub block_timestamp: i64,
}

impl ActiveModelBehavior for ActiveModel {}

impl From<crate::Attestation> for ActiveModel {
    fn from(value: crate::Attestation) -> Self {
        Self {
            index: ActiveValue::set(value.index as i64),
            root: ActiveValue::set(value.root.to_vec()),
            tx_hash: ActiveValue::set(value.tx_hash.to_vec()),
            block_number: ActiveValue::set(value.block_number as i64),
            block_timestamp: ActiveValue::set(value.block_timestamp as i64),
        }
    }
}

impl From<Model> for crate::Attestation {
    fn from(value: Model) -> Self {
        Self {
            index: value.index as u64,
            root: B256::from_slice(&value.root),
            tx_hash: TxHash::from_slice(&value.tx_hash),
            block_number: value.block_number as u64,
            block_timestamp: value.block_timestamp as u64,
        }
    }
}
