#![feature(maybe_uninit_fill)]
#![feature(likely_unlikely)]
//! High performance binary Merkle tree implementation in Rust.

use bytemuck::Pod;
use digest::{Digest, FixedOutputReset, Output};
use std::hint::unlikely;

/// Prefix byte to distinguish internal nodes from leaves when hashing.
pub const INNER_NODE_PREFIX: u8 = 0x01;

/// Flat, Fixed-Size, Read only Merkle Tree
///
/// Expects the length of leaves to be equal or near(less) to a power of two.
///
/// Leaves are **sorted** starting at index `len`.
#[derive(Debug, Clone, Default)]
pub struct FlatMerkleTree<D: Digest> {
    /// Index 0 is not used, leaves start at index `len`.
    nodes: Box<[Output<D>]>,
    len: usize,
}

/// Merkle Tree without hashing the leaves
#[derive(Debug, Clone)]
pub struct UnhashedFlatMerkleTree<D: Digest> {
    buffer: Vec<Output<D>>,
    len: usize,
}

impl<D: Digest + FixedOutputReset> FlatMerkleTree<D>
where
    Output<D>: Pod + Copy,
{
    /// Constructs a new Merkle tree from the given hash leaves.
    pub fn new(data: &[Output<D>]) -> Self {
        Self::new_unhashed(data).finalize()
    }

    /// Constructs a new Merkle tree from the given hash leaves, without hashing internal nodes.
    pub fn new_unhashed(data: &[Output<D>]) -> UnhashedFlatMerkleTree<D> {
        let raw_len = data.len();
        assert_ne!(raw_len, 0, "Cannot create Merkle tree with zero leaves");

        let len = raw_len.next_power_of_two();
        let mut nodes = Vec::<Output<D>>::with_capacity(2 * len);

        unsafe {
            let maybe_uninit = nodes.spare_capacity_mut();

            // SAFETY: tree is valid for writes, properly aligned, and at least 1 element long.
            // index 0, we will never use it
            maybe_uninit
                .get_unchecked_mut(0)
                .write(Output::<D>::default());

            // SAFETY: capacity * sizeof(T) is within the allocated size of `tree`
            let dst = maybe_uninit.get_unchecked_mut(len..).as_mut_ptr().cast();
            let src = data.as_ptr();
            // SAFETY:
            // - src is valid for reads `len` elements and properly aligned
            // - dst is valid for writes `len` elements and properly aligned
            // - the regions do not overlap since we just allocated `tree`
            std::ptr::copy_nonoverlapping(src, dst, raw_len);

            // SAFETY: capacity + len is within the allocated size of `tree`
            maybe_uninit
                .get_unchecked_mut(len + raw_len..)
                .write_filled(Output::<D>::default());

            maybe_uninit
                .get_unchecked_mut(len..)
                .assume_init_mut()
                .sort_unstable();
        }

        UnhashedFlatMerkleTree { buffer: nodes, len }
    }

    /// Returns the root hash of the Merkle tree
    #[inline]
    pub fn root(&self) -> &Output<D> {
        // SAFETY: index 1 is always initialized in new()
        unsafe { self.nodes.get_unchecked(1) }
    }

    /// Returns the leaves of the Merkle tree
    #[inline]
    pub fn leaves(&self) -> &[Output<D>] {
        unsafe { self.nodes.get_unchecked(self.len..self.len + self.len) }
    }

    /// Checks if the given leaf is contained in the Merkle tree
    #[inline]
    pub fn contains(&self, leaf: &Output<D>) -> bool {
        self.leaves().binary_search(leaf).is_ok()
    }

    /// Get proof for a given leaf
    pub fn get_proof_iter(&self, leaf: &Output<D>) -> Option<SiblingIter<'_, D>> {
        let leaf_index_in_slice = self.leaves().binary_search(leaf).ok()?;
        Some(SiblingIter {
            nodes: &self.nodes,
            current: self.len + leaf_index_in_slice,
        })
    }

    /// Returns the raw bytes of the Merkle tree nodes
    #[inline]
    pub fn as_raw_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.nodes)
    }

    /// From raw bytes, reconstruct the Merkle tree
    #[inline]
    pub unsafe fn from_raw_bytes(bytes: &[u8]) -> Self {
        let nodes: &[Output<D>] = bytemuck::cast_slice(bytes);
        let len = nodes.len() / 2;
        Self {
            nodes: nodes.to_vec().into_boxed_slice(),
            len,
        }
    }
}

impl<D: Digest + FixedOutputReset> UnhashedFlatMerkleTree<D>
where
    Output<D>: Pod + Copy,
{
    /// Finalizes the Merkle tree by hashing internal nodes
    pub fn finalize(self) -> FlatMerkleTree<D> {
        let mut nodes = self.buffer;
        let len = self.len;
        unsafe {
            let maybe_uninit = nodes.spare_capacity_mut();

            // Build the tree
            let mut hasher = D::new();
            for i in (1..len).rev() {
                // SAFETY: in bounds due to loop range and initialization above
                let left = maybe_uninit.get_unchecked(2 * i).assume_init_ref();
                let right = maybe_uninit.get_unchecked(2 * i + 1).assume_init_ref();

                Digest::update(&mut hasher, [INNER_NODE_PREFIX]);
                Digest::update(&mut hasher, left);
                Digest::update(&mut hasher, right);
                let parent_hash = hasher.finalize_reset();

                maybe_uninit.get_unchecked_mut(i).write(parent_hash);
            }

            // SAFETY: initialized all elements.
            nodes.set_len(2 * len);
        }
        FlatMerkleTree {
            nodes: nodes.into_boxed_slice(),
            len,
        }
    }
}

/// Iterator over the sibling nodes of a leaf in the Merkle tree
#[derive(Debug, Clone)]
pub struct SiblingIter<'a, D: Digest> {
    nodes: &'a [Output<D>],
    current: usize,
}

/// Indicates current node position relative to its sibling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodePosition {
    /// The sibling is a right child, `APPEND` its hash when computing the parent
    Left,
    /// The sibling is a left child, `PREPEND` its hash when computing the parent
    Right,
}

impl<'a, D: Digest> Iterator for SiblingIter<'a, D> {
    /// (Yielded Node Position, Sibling Hash)
    type Item = (NodePosition, &'a Output<D>);

    fn next(&mut self) -> Option<Self::Item> {
        if unlikely(self.current <= 1) {
            return None;
        }
        let side = if (self.current & 1) == 0 {
            NodePosition::Left
        } else {
            NodePosition::Right
        };
        let sibling_index = self.current ^ 1;
        let sibling = unsafe { self.nodes.get_unchecked(sibling_index) };
        self.current >>= 1;
        Some((side, sibling))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.current.ilog2() as usize;
        (exact, Some(exact))
    }
}

impl<D: Digest> ExactSizeIterator for SiblingIter<'_, D> {
    fn len(&self) -> usize {
        self.current.ilog2() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        test_merkle_tree::<sha2::Sha256>();
        test_merkle_tree::<sha3::Keccak256>();
    }

    #[test]
    fn proof() {
        test_proof::<sha2::Sha256>();
        test_proof::<sha3::Keccak256>();
    }

    fn test_merkle_tree<D: Digest + FixedOutputReset>()
    where
        Output<D>: Pod + Copy,
    {
        let mut leaves = vec![
            D::digest(b"leaf1"),
            D::digest(b"leaf2"),
            D::digest(b"leaf3"),
            D::digest(b"leaf4"),
        ];
        leaves.sort_unstable();

        let tree = FlatMerkleTree::<D>::new(&leaves);

        // Manually compute the expected root
        let mut hasher = D::new();
        Digest::update(&mut hasher, &leaves[0]);
        Digest::update(&mut hasher, &leaves[1]);
        let left_hash = hasher.finalize_reset();

        Digest::update(&mut hasher, &leaves[2]);
        Digest::update(&mut hasher, &leaves[3]);
        let right_hash = hasher.finalize_reset();

        Digest::update(&mut hasher, &left_hash);
        Digest::update(&mut hasher, &right_hash);
        let expected_root = hasher.finalize();

        assert_eq!(tree.root().as_slice(), expected_root.as_slice());
    }

    fn test_proof<D: Digest + FixedOutputReset>()
    where
        Output<D>: Pod + Copy,
    {
        let mut leaves = vec![
            D::digest(b"apple"),
            D::digest(b"banana"),
            D::digest(b"cherry"),
            D::digest(b"date"),
        ];
        leaves.sort_unstable();

        let tree = FlatMerkleTree::<D>::new(&leaves);

        for leaf in &leaves {
            let mut iter = tree
                .get_proof_iter(leaf)
                .expect("Leaf should be in the tree");
            let mut current_hash = *leaf;

            let mut hasher = D::new();
            while let Some((side, sibling_hash)) = iter.next() {
                match side {
                    NodePosition::Left => {
                        Digest::update(&mut hasher, &current_hash);
                        Digest::update(&mut hasher, sibling_hash);
                    }
                    NodePosition::Right => {
                        Digest::update(&mut hasher, sibling_hash);
                        Digest::update(&mut hasher, &current_hash);
                    }
                }
                current_hash = hasher.finalize_reset();
            }

            assert_eq!(current_hash.as_slice(), tree.root().as_slice());
        }
    }
}
