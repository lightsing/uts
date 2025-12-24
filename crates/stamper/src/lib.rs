#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

//! Timestamping

#[macro_use]
extern crate tracing;

use bytemuck::{NoUninit, Pod};
use digest::{Digest, FixedOutputReset, Output, typenum::Unsigned};
use rocksdb::{DB, WriteBatch};
use std::{collections::VecDeque, fmt, sync::Arc, time::Duration};
use tokio::time::{Interval, MissedTickBehavior};
use uts_bmt::FlatMerkleTree;
use uts_core::utils::Hexed;
use uts_journal::reader::JournalReader;

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
pub struct Stamper<D>
where
    D: Digest + FixedOutputReset,
    [u8; D::OutputSize::USIZE]:,
{
    /// Journal reader to read entries from
    reader: JournalReader<{ D::OutputSize::USIZE }>,
    /// Storage for merkle trees and leaf->root mappings
    storage: Arc<DB>,
    /// FIFO cache of recent merkle trees
    cache: VecDeque<FlatMerkleTree<D>>,
    /// Stamper configuration
    config: StamperConfig,
}

/// Configuration for the Stamper
#[derive(Debug, Clone)]
pub struct StamperConfig {
    /// The maximum interval (in seconds) between create new timestamps
    max_interval_seconds: u64,
    /// The maximum number of entries per timestamp.
    /// It should be a power of two.
    max_entries_per_timestamp: usize,
    /// The minimum size of the Merkle tree leaves.
    /// It should be a power of two.
    min_leaves: usize,
    /// The maximum number of recent Merkle trees to keep in cache.
    max_cache_size: usize,
}

impl<D> fmt::Debug for Stamper<D>
where
    D: Digest + FixedOutputReset,
    [u8; D::OutputSize::USIZE]:,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stamper").finish()
    }
}

impl<D> Stamper<D>
where
    D: Digest + FixedOutputReset,
    Output<D>: Pod + Copy,
    [u8; D::OutputSize::USIZE]: NoUninit,
{
    /// Work loop
    pub async fn run(&mut self) {
        let mut ticker =
            tokio::time::interval(Duration::from_secs(self.config.max_interval_seconds));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        let mut leaves_buffer = Vec::with_capacity(self.config.max_entries_per_timestamp);
        loop {
            self.pack(&mut ticker, &mut leaves_buffer).await;
        }
    }

    async fn pack(&mut self, ticker: &mut Interval, buffer: &mut Vec<[u8; D::OutputSize::USIZE]>) {
        let entries = self
            .reader
            .wait_at_least(self.config.max_entries_per_timestamp);

        let target_size = tokio::select! {
            _ = ticker.tick() => {
                // Timeout reached, create timestamp with available entries
                let current_available = self.reader.available();
                if current_available == 0 {
                    debug!("No available entries, skipping this round...");
                    return;
                }

                debug!(current_available, "Timeout reached, creating timestamp");

                // Determine the number of entries to take
                let next_power_of_two = current_available.next_power_of_two();
                if next_power_of_two == current_available {
                    trace!("Current available is power of two, taking all");
                    current_available
                } else if next_power_of_two / 2 >= self.config.min_leaves {
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
        trace!(target_size);

        // Read entries, could need two reads if wrapping around
        buffer.clear();
        buffer.extend_from_slice(self.reader.read(target_size));
        let remaining = target_size - buffer.len();
        if remaining > 0 {
            buffer.extend_from_slice(self.reader.read(remaining));
        }
        debug_assert_eq!(buffer.len(), target_size);

        let merkle_tree = FlatMerkleTree::<D>::new_unhashed(bytemuck::cast_slice(&buffer));
        let storage = self.storage.clone();

        let merkle_tree = tokio::task::spawn_blocking(move || {
            let merkle_tree = merkle_tree.finalize(); // CPU intensive
            let root = merkle_tree.root();
            info!(root = ?Hexed(root));

            let mut batch = WriteBatch::default();
            batch.put(root, merkle_tree.as_raw_bytes());
            for leaf in merkle_tree.leaves() {
                batch.put(leaf, root);
            }
            storage.write(batch).expect("Failed to write to storage"); // FIXME: handle error properly
            merkle_tree
        })
        .await
        .expect("Failed to create Merkle tree"); // FIXME: handle error properly

        if self.cache.len() >= self.config.max_cache_size {
            self.cache.pop_front();
        }
        self.cache.push_back(merkle_tree);

        self.reader.commit();
    }
}
