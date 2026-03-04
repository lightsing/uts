use alloy_primitives::B256;
use bytemuck::Pod;
use digest::{Digest, FixedOutputReset, Output};
use rocksdb::DB;
use uts_bmt::MerkleTree;

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
pub trait DbExt<D: Digest> {
    /// Load a Merkle entry from the database by root hash
    fn load_trie(&self, root: B256) -> Result<Option<MerkleTree<D>>, StorageError>;

    /// Get the root hash for a given leaf hash, if it exists
    fn get_root_for_leaf(&self, leaf: B256) -> Result<Option<B256>, StorageError>;
}

impl<D: Digest + FixedOutputReset> DbExt<D> for DB
where
    Output<D>: Pod + Copy,
{
    fn load_trie(&self, root: B256) -> Result<Option<MerkleTree<D>>, StorageError> {
        let Some(data) = self.get(root)? else {
            return Ok(None);
        };
        Ok(Some(MerkleTree::from_raw_bytes(&data)))
    }

    fn get_root_for_leaf(&self, leaf: B256) -> Result<Option<B256>, StorageError> {
        let Some(root) = self.get(leaf)? else {
            return Ok(None);
        };
        if root.len() != 32 {
            let trie = MerkleTree::<D>::from_raw_bytes(&root);
            debug_assert_eq!(trie.root().as_slice(), leaf);
            debug_assert_eq!(trie.leaves().len(), 1);
            return Ok(Some(leaf)); // it's a single-leaf tree
        }
        let hash: [u8; 32] = root.as_slice().try_into().expect("infallible");
        Ok(Some(B256::new(hash)))
    }
}
