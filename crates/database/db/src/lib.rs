//! Library to interact with database.

use alloy_primitives::{B256, BlockNumber, BlockTimestamp, TxHash};
use sea_orm::{DbErr, prelude::async_trait};

mod models;
use models::attestation::Model as AttestationModel;

/// UTS attestation.
#[derive(Debug)]
pub struct Attestation {
    /// Incremental index for an attestation.
    pub index: u64,
    /// The merkle tree root to be attested.
    pub root: B256,
    /// Tx hash of on-chain attestation.
    pub tx_hash: TxHash,
    /// Block number of the block in which on-chain attestation was included.
    pub block_number: BlockNumber,
    /// Timestamp of the block in which on-chain attestation was included.
    pub block_timestamp: BlockTimestamp,
}

/// Database operations related to attestations.
#[async_trait::async_trait]
pub trait Database: Send + Sync {
    /// Create a new attestation.
    async fn create_attestation(
        &self,
        root: B256,
        tx_hash: TxHash,
    ) -> Result<AttestationModel, DbErr>;

    /// Update an existing attestation.
    async fn update_attestation(
        &self,
        root: B256,
        block_number: BlockNumber,
        block_timestamp: BlockTimestamp,
    ) -> Result<AttestationModel, DbErr>;

    /// Delete an attestation by root.
    async fn delete_attestation_by_root(&self, root: B256) -> Result<(), DbErr>;

    /// Delete an attestation by tx hash.
    async fn delete_attestation_by_tx_hash(&self, tx_hash: TxHash) -> Result<(), DbErr>;
}
