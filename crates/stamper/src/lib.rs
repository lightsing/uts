#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

//! Timestamping

#[macro_use]
extern crate tracing;

use alloy_primitives::B256;
use alloy_provider::Provider;
use bytemuck::{NoUninit, Pod};
use digest::{Digest, FixedOutputReset, Output, typenum::Unsigned};
use eyre::Context;
use rocksdb::{DB, WriteBatch};
use sqlx::SqlitePool;
use std::{fmt, marker::PhantomData, sync::Arc, time::Duration};
use tokio::{
    select,
    time::{Interval, MissedTickBehavior},
};
use tokio_util::sync::CancellationToken;
use uts_bmt::MerkleTree;
use uts_contracts::eas::EAS;
use uts_core::utils::Hexed;
use uts_journal::reader::JournalReader;

/// kv storage for leaf->root mappings, and root->full_tree mappings (for power-of-two sized trees)
pub mod kv;
/// sql storage for pending attestation metadata, and finalized timestamp metadata
pub mod sql;

mod tx_sender;

/// Maximum number of retries for transient errors in transaction sending.
pub const MAX_RETRIES: usize = 3;

/// Stamper for timestamping
///
/// A stamper will wait for, either:
/// - Timeout: `max_interval_seconds` has passed since last timestamp
/// - Max Entries: `max_entries_per_timestamp` have been collected since last timestamp
///
/// Then it will collect entries from the journal reader, with the size of:
/// - at most `max_entries_per_timestamp`
/// - if available entries size is not power of two, it will take:
///  - the largest power of two less than available entries, if that is >= `min_leaves`
///  - else, it will take all available entries
pub struct Stamper<D: Digest, P, const ENTRY_SIZE: usize> {
    /// Journal reader to read entries from
    reader: JournalReader<ENTRY_SIZE>,
    /// Storage for merkle trees and leaf->root mappings
    kv_storage: Arc<DB>,
    /// Sql db for storing timestamp metadata
    sql_storage: SqlitePool,

    /// The contract
    contract: EAS<P>,
    /// Stamper configuration
    config: StamperConfig,

    _marker: PhantomData<D>,
}

/// Configuration for the Stamper
#[derive(Debug, Clone)]
pub struct StamperConfig {
    /// The maximum interval (in seconds) between create new timestamps
    pub max_interval_seconds: u64,
    /// The maximum number of entries per timestamp.
    /// It should be a power of two.
    pub max_entries_per_timestamp: usize,
    /// The minimum size of the Merkle tree leaves.
    /// It should be a power of two.
    pub min_leaves: usize,
    /// The maximum number of recent Merkle trees to keep in cache.
    pub max_cache_size: usize,
}

impl<D: Digest, P, const ENTRY_SIZE: usize> fmt::Debug for Stamper<D, P, ENTRY_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stamper")
            .field("config", &self.config)
            .finish()
    }
}

impl<D, P, const ENTRY_SIZE: usize> Stamper<D, P, ENTRY_SIZE>
where
    D: Digest + FixedOutputReset + 'static,
    P: Provider + Clone + 'static,
    Output<D>: Pod + Copy,
    [u8; ENTRY_SIZE]: NoUninit,
{
    const _SIZE_MATCHES: () = assert!(D::OutputSize::USIZE == ENTRY_SIZE);

    /// Create a new Stamper
    pub fn new(
        reader: JournalReader<ENTRY_SIZE>,
        kv_storage: Arc<DB>,
        sql_storage: SqlitePool,
        contract: EAS<P>,
        config: StamperConfig,
    ) -> Self {
        Self {
            reader,
            kv_storage,
            sql_storage,
            contract,
            config,
            _marker: PhantomData,
        }
    }

    /// Work loop
    pub async fn run(&mut self, token: CancellationToken) -> eyre::Result<()> {
        let mut ticker =
            tokio::time::interval(Duration::from_secs(self.config.max_interval_seconds));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        let mut leaves_buffer = Vec::with_capacity(self.config.max_entries_per_timestamp);
        let (waker_tx, waker_rx) = tokio::sync::mpsc::channel::<()>(1);

        let tx_sender = tx_sender::TxSender {
            eas: self.contract.clone(),
            sql_storage: self.sql_storage.clone(),
            waker: waker_rx,
            token: token.clone(),
        };
        tokio::spawn(async move {
            tx_sender.run_until_cancelled().await;
        });

        loop {
            select! {
                _ = token.cancelled() => {
                    info!("Cancellation received, stopping stamper...");
                    break;
                }
                res = self.pack(&mut ticker, &mut leaves_buffer) => {
                    match res {
                        Err(e) => {
                            error!(error = ?e, "Error in packing batch, stopping stamper...");
                            break;
                        }
                        Ok(0) => continue,
                        // notify the tx sender to wake up
                        Ok(e) => {
                            info!(entries = e, "Batch packed successfully, waking up TxSender...");
                            waker_tx.try_send(()).ok()
                        },
                    };
                }
            }
        }
        Ok(())
    }

    /// Try to pack a batch of entries into a Merkle tree and push the pending attestation to SQL db.
    ///
    /// # Errors
    ///
    /// All errors here are considered **FATAL**, meaning the stamper cannot continue to run correctly,
    /// and should be restarted after the error is resolved.
    ///
    /// The errors likely indicate an issue with the underlying storage (either journal or db),
    /// and should be resolved by fixing the storage issue (e.g. clearing corrupted data,
    /// increasing disk space, etc.)
    async fn pack(
        &mut self,
        ticker: &mut Interval,
        buffer: &mut Vec<[u8; ENTRY_SIZE]>,
    ) -> eyre::Result<usize> {
        let target_size = self.wait_for_next_batch(ticker).await;
        // no entries to process, skip this round
        if target_size == 0 {
            return Ok(0);
        };
        trace!(target_size);

        // Read entries, could need two reads if wrapping around
        buffer.clear();
        buffer.extend_from_slice(self.reader.read(target_size)?);
        debug_assert_eq!(buffer.len(), target_size);

        let merkle_tree = MerkleTree::<D>::new_unhashed(bytemuck::cast_slice(buffer));

        let merkle_tree = tokio::task::spawn_blocking(move || {
            let merkle_tree = merkle_tree.finalize(); // CPU intensive
            let root = merkle_tree.root();
            info!(root = ?Hexed(root));
            merkle_tree
        })
        .await
        .context("failed to create Merkle tree")?;

        let root = B256::new(bytemuck::cast(*merkle_tree.root()));

        let mut batch = WriteBatch::default();
        // store leaf->root mappings for quick lookup
        for leaf in merkle_tree.leaves() {
            batch.put(leaf, root);
        }
        // if it's a single-leaf tree, the root == the leaf, so we write mapping first.
        batch.put(root, merkle_tree.as_raw_bytes());
        self.kv_storage
            .write(batch)
            .context("failed to write to kv db")?;

        sql::new_pending_attestation(&self.sql_storage, root)
            .await
            .context("failed to create pending attestation in sql db")?;

        self.reader
            .commit()
            .context("failed to commit journal reader")?;
        Ok(target_size)
    }

    async fn wait_for_next_batch(&mut self, ticker: &mut Interval) -> usize {
        let entries = self
            .reader
            .wait_at_least(self.config.max_entries_per_timestamp);

        let target_size = select! {
            _ = ticker.tick() => {
                // Timeout reached, create timestamp with available entries
                let current_available = self.reader.available();
                if current_available == 0 {
                    debug!("No available entries, skipping this round...");
                    return 0;
                }

                debug!(current_available, "Timeout reached, creating timestamp");

                // Determine the number of entries to take
                let next_power_of_two = current_available.next_power_of_two();
                if next_power_of_two == current_available {
                    trace!("Current available is power of two, taking all");
                    current_available
                } else if next_power_of_two / 2 >= self.config.min_leaves {
                    // This is for avoiding creating small Merkle trees with too few leaves.
                    // e.g. if current_available = 3, min_leaves = 4 then next_power_of_two / 2 = 2,
                    // we will take 3 entries instead of 2 to create a Merkle tree with 4 leaves instead of 1 leaf.
                    let target = next_power_of_two / 2;
                    trace!(target, "Taking largest power of two less than available");
                    target
                } else {
                    trace!("Taking all available entries");
                    current_available
                }
            }
            _ = entries => {
                // Max entries reached, create timestamp
                debug!("Max entries reached, creating timestamp");
                self.config.max_entries_per_timestamp
            }
        };
        target_size
    }
}
