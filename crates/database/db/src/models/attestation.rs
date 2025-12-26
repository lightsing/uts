//! Attestation model.

use crate::types::Wrapped;
use alloy_primitives::{B256, BlockNumber, BlockTimestamp, TxHash};
use sea_orm::{ActiveValue, entity::prelude::*};

/// Database model representing a UTS Attestation.
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[sea_orm(table_name = "attestation")]
pub struct Model {
    /// Incremental index for entries into the table.
    #[sea_orm(primary_key)]
    pub index: i64,
    /// The merkle tree root to attest.
    #[sea_orm(unique)]
    pub root: Wrapped<B256>,
    /// The tx hash for the on-chain attestation.
    #[sea_orm(unique)]
    pub tx_hash: Wrapped<TxHash>,
    /// The block number in which the on-chain attestation was included.
    ///
    /// The "attestation" table is indexed by this column.
    #[sea_orm(indexed, nullable)]
    pub block_number: Option<i64>,
    /// The timestamp for the block in which the on-chain attestation was included.
    ///
    /// The "attestation" table is indexed by this column.
    #[sea_orm(indexed, nullable)]
    pub block_timestamp: Option<i64>,
}

impl ActiveModelBehavior for ActiveModel {}

impl From<crate::Attestation> for ActiveModel {
    fn from(value: crate::Attestation) -> Self {
        Self {
            index: ActiveValue::set(value.index as i64),
            root: ActiveValue::set(Wrapped(value.root)),
            tx_hash: ActiveValue::set(Wrapped(value.tx_hash)),
            block_number: ActiveValue::set(value.block_number.map(|n| n as i64)),
            block_timestamp: ActiveValue::set(value.block_timestamp.map(|t| t as i64)),
        }
    }
}

impl From<Model> for crate::Attestation {
    fn from(value: Model) -> Self {
        Self {
            index: value.index as u64,
            root: value.root.into_inner(),
            tx_hash: value.tx_hash.into_inner(),
            block_number: value.block_number.map(|n| n as BlockNumber),
            block_timestamp: value.block_timestamp.map(|t| t as BlockTimestamp),
        }
    }
}
