use crate::{
    config::RelayerConfig,
    sql,
    sql::l1_batch::{L1Batch, L1BatchStatus},
};
use alloy_primitives::{B256, ChainId, U256};
use alloy_provider::Provider;
use eyre::bail;
use jiff::Timestamp;
use sha3::{Keccak256, digest::Output};
use sqlx::SqlitePool;
use std::time::Duration;
use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;
use uts_bmt::MerkleTree;
use uts_contracts::{gateway::L1AnchoringGateway, manager::L2AnchoringManager};

/// Relayer responsible for packing L2 anchoring requests into batches, submitting them to L1, and finalizing them on L2.
#[derive(Debug)]
pub struct Relayer<P1, P2> {
    db: SqlitePool,
    l2_chain_id: ChainId,
    gateway: L1AnchoringGateway<P1>,
    manager: L2AnchoringManager<P2>,
    config: RelayerConfig,
    cancellation_token: CancellationToken,
}

impl<P1: Provider, P2: Provider> Relayer<P1, P2> {
    /// Create a new instance of the relayer.
    pub async fn new(
        db: SqlitePool,
        gateway: L1AnchoringGateway<P1>,
        manager: L2AnchoringManager<P2>,
        config: RelayerConfig,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<Self> {
        let l2_chain_id = manager.provider().get_chain_id().await?;
        Ok(Self {
            db,
            l2_chain_id,
            gateway,
            manager,
            config,
            cancellation_token,
        })
    }

    /// Run the relayer loop until cancellation is requested.
    pub async fn run(self) -> eyre::Result<()> {
        loop {
            select! {
                may_err = self.tick() => {
                    if let Err(e) = may_err {
                        error!("Error in relayer tick: {e:?}");
                        self.cancellation_token.cancel();
                        return Err(e)
                    }
                }
                _ = self.cancellation_token.cancelled() => {
                    info!("Cancellation requested, stopping relayer");
                    break;
                }
            }
            sleep(Duration::from_secs(self.config.tick_interval_seconds)).await;
        }
        Ok(())
    }

    async fn tick(&self) -> eyre::Result<()> {
        let latest_batch = sql::l1_batch::get_latest_l1batch(&self.db).await?;

        match latest_batch {
            None
            | Some(L1Batch {
                status: L1BatchStatus::L2Finalized,
                ..
            }) => {
                self.may_pack_new_batch(latest_batch).await?;
            }
            Some(batch) if batch.status == L1BatchStatus::L1Sent => {
                debug!(
                    "Batch {} is in flight. Waiting for Indexer to mark it ARRIVED...",
                    batch.id
                );
            }
            Some(batch) if batch.status == L1BatchStatus::L2Received => {
                self.finalize_batch(batch).await?;
            }
            Some(batch) if batch.status == L1BatchStatus::L2Finalizing => {
                debug!(
                    "Batch {} is finalizing on L2. Waiting for Indexer to mark it FINALIZED...",
                    batch.id
                );
            }
            _ => bail!("unreachable state"),
        }
        Ok(())
    }

    async fn may_pack_new_batch(&self, last_batch: Option<L1Batch>) -> eyre::Result<()> {
        // 1. Calculate the next batch's start index
        let next_start_index = last_batch.map(|b| b.start_index + b.count).unwrap_or(1);

        // 2. Check required conditions for packing a new batch
        let pending_counts =
            sql::anchoring_requests::count_pending_events(&self.db, next_start_index).await?;
        if pending_counts == 0 {
            return Ok(());
        }

        let now = Timestamp::now();
        let last_batch_time = last_batch.map(|b| b.updated_at).unwrap_or(Timestamp::MIN);
        let elapsed = now.duration_since(last_batch_time).as_secs();

        if pending_counts < self.config.batch_max_size
            && elapsed < self.config.batch_max_wait_seconds
        {
            return Ok(());
        }

        let counts = pending_counts.min(self.config.batch_max_size);

        info!(
            "Sealing a new batch starting from index {next_start_index} with {counts} pending events (elapsed: {elapsed}s)"
        );
        let leaves =
            sql::anchoring_requests::load_roots_in_range(&self.db, next_start_index, counts)
                .await?
                .into_iter()
                .map(|hash| Output::<Keccak256>::from(hash.0))
                .collect::<Vec<_>>();
        let trie = MerkleTree::<Keccak256>::new(&leaves);
        let root = B256::new(trie.root().0);

        sql::l1_batch::upsert_l1batch(
            &self.db,
            self.l2_chain_id,
            root,
            next_start_index,
            counts,
            L1BatchStatus::L1Sent,
        )
        .await?;

        let tx_hash = self
            .gateway
            .submitBatch(
                root,
                U256::from(next_start_index),
                U256::from(counts),
                U256::from(self.config.l1_batch_submission_gas_limit),
            )
            .send()
            .await?
            .watch()
            .await?;
        info!("Batch submission transaction sent with hash: {tx_hash}");

        Ok(())
    }

    async fn finalize_batch(&self, batch: L1Batch) -> eyre::Result<()> {
        info!(
            "Batch {} arrived on L2. Submitting finalization tx...",
            batch.id
        );

        sql::l1_batch::update_l1batch_status(&self.db, batch.id, L1BatchStatus::L2Finalizing)
            .await?;

        let tx_hash = self.manager.finalizeBatch().send().await?.watch().await?;
        info!("Finalize transaction sent with hash: {}", tx_hash);

        Ok(())
    }
}
