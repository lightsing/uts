use crate::{
    config::RelayerConfig,
    sql,
    sql::{
        eth::insert_tx_receipt,
        l1_batch::{L1Batch, L1BatchStatus},
    },
};
use alloy_primitives::{B256, ChainId, U256};
use alloy_provider::{PendingTransactionBuilder, Provider};
use eyre::{Context, ContextCompat, bail};
use jiff::Timestamp;
use sha3::{Keccak256, digest::Output};
use sqlx::SqlitePool;
use std::time::Duration;
use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;
use uts_bmt::MerkleTree;
use uts_contracts::{
    eas::events::Timestamped,
    gateway::L1AnchoringGateway,
    manager::{L2AnchoringManager, events::L1BatchFinalized},
};

/// Relayer responsible for packing L2 anchoring requests into batches, submitting them to L1, and finalizing them on L2.
#[derive(Debug)]
pub struct Relayer<P1, P2> {
    db: SqlitePool,
    l1_chain_id: ChainId,
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
        let l1_chain_id = gateway.provider().get_chain_id().await?;
        let l2_chain_id = manager.provider().get_chain_id().await?;
        Ok(Self {
            db,
            l1_chain_id,
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
                    match may_err {
                        Ok(should_tick_immediately) => {
                            if should_tick_immediately {
                                continue;
                            }
                            sleep(Duration::from_secs(self.config.tick_interval_seconds)).await;
                        }
                        Err(e) => {
                            error!("Error in relayer tick: {e:?}");
                            self.cancellation_token.cancel();
                            return Err(e)
                        }
                    }
                }
                _ = self.cancellation_token.cancelled() => {
                    info!("Cancellation requested, stopping relayer");
                    break;
                }
            }
        }
        Ok(())
    }

    /// Perform one tick of the relayer loop.
    ///
    /// # Returns
    ///
    /// Whether you should immediately tick again without waiting for the next interval.
    async fn tick(&self) -> eyre::Result<bool> {
        let latest_batch = sql::l1_batch::get_latest_l1batch(&self.db).await?;

        match latest_batch {
            None
            | Some(L1Batch {
                status: L1BatchStatus::L2Finalized,
                ..
            }) => self.may_pack_new_batch(latest_batch).await,
            Some(batch) if batch.status == L1BatchStatus::Collected => {
                self.send_attest_tx(batch).await?;
                Ok(true)
            }
            Some(batch) if batch.status == L1BatchStatus::L1Sent => {
                self.watch_l1_tx(batch).await?;
                Ok(true)
            }
            Some(batch) if batch.status == L1BatchStatus::L1Mined => {
                trace!(
                    "Batch {} is mined but not yet received on L2. Waiting for it to arrive...",
                    batch.id
                );
                Ok(false)
            }
            Some(batch) if batch.status == L1BatchStatus::L2Received => {
                self.send_finalize_batch_tx(batch).await?;
                Ok(true)
            }
            Some(batch) if batch.status == L1BatchStatus::L2FinalizeTxSent => {
                self.watch_finalize_batch_tx(batch).await?;
                Ok(true)
            }
            _ => bail!("unreachable state"),
        }
    }

    /// Watch the L1 transaction for the given batch until it's mined, and update the batch status accordingly.
    ///
    /// # Returns
    ///
    /// Returns true if a new batch if packed and ready to be sent, so the relayer should tick again
    /// immediately without waiting for the next interval.
    #[instrument(skip_all, err)]
    async fn may_pack_new_batch(&self, last_batch: Option<L1Batch>) -> eyre::Result<bool> {
        trace!("Checking if we can pack a new batch...");

        // 1. Calculate the next batch's start index
        let next_start_index = last_batch.map(|b| b.start_index + b.count).unwrap_or(1);

        // 2. Check required conditions for packing a new batch
        let pending_counts =
            sql::anchoring_requests::count_pending_events(&self.db, next_start_index).await?;
        if pending_counts == 0 {
            return Ok(false);
        }

        let now = Timestamp::now();
        let last_batch_time = last_batch.map(|b| b.updated_at).unwrap_or(Timestamp::MIN);
        let elapsed = now.duration_since(last_batch_time).as_secs();

        if pending_counts < self.config.batch_max_size
            && elapsed < self.config.batch_max_wait_seconds
        {
            return Ok(false);
        }

        let counts = pending_counts.min(self.config.batch_max_size);

        info!(
            start_index = next_start_index,
            counts,
            max_size_reached = pending_counts >= self.config.batch_max_size,
            max_wait_reached = elapsed >= self.config.batch_max_wait_seconds,
            "Sealing a new batch"
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
            L1BatchStatus::Collected,
        )
        .await?;

        Ok(true)
    }

    #[instrument(skip_all, fields(batch_id = batch.id), err)]
    async fn send_attest_tx(&self, batch: L1Batch) -> eyre::Result<()> {
        let pending_tx = self
            .gateway
            .submitBatch(
                batch.root,
                U256::from(batch.start_index),
                U256::from(batch.count),
                U256::from(self.config.l1_batch_submission_gas_limit),
            )
            .value(self.config.l1_batch_submission_fee)
            .send()
            .await?;

        let tx_hash = pending_tx.tx_hash();
        info!(%tx_hash, "Batch submission transaction broadcasted");
        sql::l1_batch::update_l1batch_to_l1_sent(&self.db, batch.id, *tx_hash).await?;

        Ok(())
    }

    #[instrument(skip_all, fields(batch_id = batch.id), err)]
    async fn watch_l1_tx(&self, batch: L1Batch) -> eyre::Result<()> {
        let tx_hash = batch
            .l1_tx_hash
            .context("Batch is in L1Sent status but missing tx hash")?;

        info!(%tx_hash, "Watching for batch submission transaction to be mined...");

        let receipt =
            PendingTransactionBuilder::new(self.gateway.provider().root().clone(), tx_hash)
                .get_receipt()
                .await
                .context("get receipt")?;
        info!(%tx_hash, block_number = receipt.block_number, "Batch submission transaction mined");

        // sanity check: make sure the transaction was successful
        if !receipt.status() {
            bail!("Batch submission transaction reverted");
        }
        if receipt.decoded_log::<Timestamped>().is_none() {
            warn!(
                "No Timestamped event found, this means the merkle root is timestamped by others before"
            );
        }

        let mut db_tx = self.db.begin().await?;
        sql::l1_batch::compare_and_set_l1batch_status(
            &mut *db_tx,
            batch.id,
            L1BatchStatus::L1Sent,
            L1BatchStatus::L1Mined,
        )
        .await?;
        insert_tx_receipt(&mut *db_tx, self.l1_chain_id, &receipt).await?;
        db_tx.commit().await?;

        Ok(())
    }

    #[instrument(skip_all, fields(batch_id = batch.id), err)]
    async fn send_finalize_batch_tx(&self, batch: L1Batch) -> eyre::Result<()> {
        info!(
            "Batch {} arrived on L2. Submitting finalization tx...",
            batch.id
        );

        let pending_tx = self.manager.finalizeBatch().send().await?;
        let tx_hash = *pending_tx.tx_hash();

        sql::l1_batch::update_l1batch_to_l2finalize_tx_sent(&self.db, batch.id, tx_hash).await?;
        info!(%tx_hash, "Finalize transaction broadcasted");

        Ok(())
    }

    #[instrument(skip_all, fields(batch_id = batch.id), err)]
    async fn watch_finalize_batch_tx(&self, batch: L1Batch) -> eyre::Result<()> {
        let tx_hash = batch
            .l2_tx_hash
            .context("Batch is in L2FinalizeTxSent status but missing L2 tx hash")?;

        info!(%tx_hash, "Watching for finalize transaction to be mined...");

        let receipt =
            PendingTransactionBuilder::new(self.manager.provider().root().clone(), tx_hash)
                .get_receipt()
                .await
                .context("get receipt")?;
        let block_number = receipt
            .block_number
            .context("missing block number in log")?;
        info!(%tx_hash, block_number, "Finalize transaction mined");

        let mut db_tx = self.db.begin().await?;
        insert_tx_receipt(&mut *db_tx, self.l2_chain_id, &receipt).await?;
        db_tx.commit().await?;

        // sanity check: make sure the transaction was successful
        if !receipt.status() {
            bail!("Finalize transaction reverted");
        }
        if receipt.decoded_log::<L1BatchFinalized>().is_none() {
            bail!("No L1BatchFinalized event found in finalize transaction receipt");
        };

        Ok(())
    }
}
