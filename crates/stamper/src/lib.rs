#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

//! Timestamping

#[macro_use]
extern crate tracing;

use alloy_primitives::{B256, BlockNumber, ChainId, TxHash};
use alloy_provider::Provider;
use bytemuck::{NoUninit, Pod};
use digest::{Digest, FixedOutputReset, Output, typenum::Unsigned};
use rocksdb::{DB, WriteBatch};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    fmt,
    sync::Arc,
    time::Duration,
};
use tokio::time::{Interval, MissedTickBehavior};
use uts_bmt::MerkleTree;
use uts_contracts::eas::IEAS::IEASInstance;
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
pub struct Stamper<D: Digest, P, const ENTRY_SIZE: usize> {
    /// Journal reader to read entries from
    reader: JournalReader<ENTRY_SIZE>,
    /// Storage for merkle trees and leaf->root mappings
    storage: Arc<DB>,
    /// FIFO cache of recent merkle trees
    cache: VecDeque<MerkleTree<D>>,
    /// FIFO cache index of recent merkle trees
    cache_index: HashMap<B256, usize>,
    /// The contract
    contract: IEASInstance<P>,
    /// Stamper configuration
    config: StamperConfig,
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

/// Merkle entry stored in the database
#[derive(Debug, Serialize, Deserialize)]
pub struct MerkleEntry<'a> {
    /// Chain ID of the timestamp transaction
    pub chain_id: ChainId,
    /// Transaction hash of the timestamp transaction
    pub tx_hash: TxHash,
    /// Block number of the timestamp transaction
    pub height: BlockNumber,

    trie: Cow<'a, [u8]>,
}

impl MerkleEntry<'_> {
    /// Get the Merkle tree from the entry
    pub fn trie<D>(&self) -> MerkleTree<D>
    where
        D: Digest + FixedOutputReset,
        Output<D>: Pod + Copy,
    {
        // SAFETY: We trust that the data in the database is valid, and that the trie was serialized correctly.
        unsafe { MerkleTree::from_raw_bytes(&self.trie) }
    }
}

/// Errors that can occur during storage operations
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// Errors from RocksDB
    #[error(transparent)]
    Rocks(#[from] rocksdb::Error),
    /// Errors from bitcode serialization/deserialization
    #[error(transparent)]
    Bitcode(#[from] bitcode::Error),
}

/// Extension trait for DB to load Merkle entries and leaf->root mappings
pub trait DbExt {
    /// Load a Merkle entry from the database by root hash
    fn load_entry(&self, root: B256) -> Result<Option<MerkleEntry<'static>>, StorageError>;

    /// Get the root hash for a given leaf hash, if it exists
    fn get_root_for_leaf(&self, leaf: B256) -> Result<Option<B256>, StorageError>;
}

impl DbExt for DB {
    fn load_entry(&self, root: B256) -> Result<Option<MerkleEntry<'static>>, StorageError> {
        let Some(data) = self.get(root)? else {
            return Ok(None);
        };
        let entry: MerkleEntry<'static> = bitcode::deserialize(&data)?;
        Ok(Some(entry))
    }

    fn get_root_for_leaf(&self, leaf: B256) -> Result<Option<B256>, StorageError> {
        let Some(root) = self.get(leaf)? else {
            return Ok(None);
        };
        if root.len() != 32 {
            let _entry: MerkleEntry<'static> = bitcode::deserialize(&root)?;
            return Ok(Some(leaf)); // it's a single-leaf tree
        }
        let hash: [u8; 32] = root.as_slice().try_into().expect("infallible");
        Ok(Some(B256::new(hash)))
    }
}

impl<D: Digest, P, const ENTRY_SIZE: usize> fmt::Debug for Stamper<D, P, ENTRY_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stamper")
            .field("cache_size", &self.cache.len())
            .field("config", &self.config)
            .finish()
    }
}

impl<D, P, const ENTRY_SIZE: usize> Stamper<D, P, ENTRY_SIZE>
where
    D: Digest + FixedOutputReset + 'static,
    P: Provider,
    Output<D>: Pod + Copy,
    [u8; ENTRY_SIZE]: NoUninit,
{
    const _SIZE_MATCHES: () = assert!(D::OutputSize::USIZE == ENTRY_SIZE);

    /// Create a new Stamper
    pub fn new(
        reader: JournalReader<ENTRY_SIZE>,
        storage: Arc<DB>,
        contract: IEASInstance<P>,
        config: StamperConfig,
    ) -> Self {
        Self {
            reader,
            storage,
            cache: VecDeque::with_capacity(config.max_cache_size),
            cache_index: HashMap::with_capacity(config.max_cache_size),
            contract,
            config,
        }
    }

    /// Work loop
    pub async fn run(&mut self) {
        let chain_id = self
            .contract
            .provider()
            .get_chain_id()
            .await
            .expect("Failed to get chain ID");
        let mut ticker =
            tokio::time::interval(Duration::from_secs(self.config.max_interval_seconds));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        let mut leaves_buffer = Vec::with_capacity(self.config.max_entries_per_timestamp);
        loop {
            self.pack(chain_id, &mut ticker, &mut leaves_buffer).await;
        }
    }

    async fn pack(
        &mut self,
        chain_id: ChainId,
        ticker: &mut Interval,
        buffer: &mut Vec<[u8; ENTRY_SIZE]>,
    ) {
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

        let merkle_tree = MerkleTree::<D>::new_unhashed(bytemuck::cast_slice(&buffer));
        let storage = self.storage.clone();

        let merkle_tree = tokio::task::spawn_blocking(move || {
            let merkle_tree = merkle_tree.finalize(); // CPU intensive
            let root = merkle_tree.root();
            info!(root = ?Hexed(root));
            merkle_tree
        })
        .await
        .expect("Failed to create Merkle tree"); // FIXME: handle error properly

        let root = B256::new(bytemuck::cast(*merkle_tree.root()));

        // commit to blockchain
        let receipt = self
            .contract
            .timestamp(root)
            .send()
            .await
            .expect("failed to build transaction")
            .get_receipt()
            .await
            .expect("failed to send transaction"); // FIXME: handle error properly
        let block_number = receipt.block_number.expect("Transaction not yet mined");
        info!(%block_number, %receipt.transaction_hash, %root,"Timestamp attested on-chain");

        // write to storage
        let mut batch = WriteBatch::default();
        // store leaf->root mappings for quick lookup
        for leaf in merkle_tree.leaves() {
            batch.put(leaf, root);
        }
        // store the Merkle tree
        let entry = MerkleEntry {
            chain_id,
            tx_hash: receipt.transaction_hash,
            height: block_number,
            trie: Cow::Borrowed(merkle_tree.as_raw_bytes()),
        };
        let serialized_entry =
            bitcode::serialize(&entry).expect("Failed to serialize Merkle entry");
        bitcode::deserialize::<MerkleEntry>(&serialized_entry)
            .expect("Failed to deserialize Merkle entry"); // sanity check
        // if it's a single-leaf tree, the root == the leaf, so we write mapping first.
        batch.put(root, serialized_entry);
        storage.write(batch).expect("Failed to write to storage"); // FIXME: handle error properly

        if self.cache.len() >= self.config.max_cache_size {
            let evicted = self
                .cache
                .pop_front()
                .expect("infallible due to check above");

            let root = evicted.root();
            let removed = self.cache_index.remove(&B256::new(bytemuck::cast(*root)));
            debug_assert_eq!(removed, Some(0));

            self.cache_index.iter_mut().for_each(|(_, idx)| *idx -= 1);
        }
        self.cache.push_back(merkle_tree);
        self.cache_index.insert(root, self.cache.len() - 1);
        self.reader.commit();
    }
}
