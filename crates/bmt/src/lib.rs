//! High performance binary Merkle tree implementation in Rust.

// MIT License
//
// Copyright (c) 2025 UTS Contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// Apache License, Version 2.0
//
// Copyright (c) 2025 UTS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use digest::{Digest, FixedOutputReset, Output, typenum::Unsigned};

/// Prefix byte to distinguish internal nodes from leaves when hashing.
pub const INNER_NODE_PREFIX: u8 = 0x01;

/// Flat, Fixed-Size, Read only Merkle Tree
///
/// Expects the length of leaves to be equal or near(less) to a power of two.
#[derive(Debug, Clone, Default)]
pub struct MerkleTree<D: Digest> {
    /// Index 0 is not used, leaves start at index `len`.
    nodes: Box<[Output<D>]>,
    len: usize,
}

/// Merkle Tree without hashing the leaves
#[derive(Debug, Clone)]
pub struct UnhashedMerkleTree<D: Digest> {
    buffer: Vec<Output<D>>,
    len: usize,
}

impl<D: Digest + FixedOutputReset> MerkleTree<D>
where
    Output<D>: Copy,
{
    /// Constructs a new Merkle tree from the given hash leaves.
    pub fn new(data: &[Output<D>]) -> Self {
        Self::new_unhashed(data).finalize()
    }

    /// Constructs a new Merkle tree from the given hash leaves, without hashing internal nodes.
    pub fn new_unhashed(data: &[Output<D>]) -> UnhashedMerkleTree<D> {
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
            for e in maybe_uninit.get_unchecked_mut(len + raw_len..) {
                e.write(Output::<D>::default());
            }
        }

        UnhashedMerkleTree { buffer: nodes, len }
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
        self.leaves().contains(leaf)
    }

    /// Get proof for a given leaf
    pub fn get_proof_iter(&self, leaf: &Output<D>) -> Option<SiblingIter<'_, D>> {
        let leaf_index_in_slice = self.leaves().iter().position(|a| a == leaf)?;
        Some(SiblingIter {
            nodes: &self.nodes,
            current: self.len + leaf_index_in_slice,
        })
    }

    /// Returns the raw bytes of the Merkle tree nodes
    #[inline]
    pub fn to_raw_bytes(&self) -> Vec<u8> {
        self.nodes
            .iter()
            .flat_map(|node| node.as_slice())
            .copied()
            .collect()
    }

    /// From raw bytes, reconstruct the Merkle tree
    ///
    /// # Panics
    ///
    /// - If the length of `bytes` is not a multiple of the hash output size.
    /// - If the number of nodes implied by `bytes` is not consistent with a valid
    ///   Merkle tree structure.
    #[inline]
    pub fn from_raw_bytes(bytes: &[u8]) -> Self {
        assert!(
            bytes.len().is_multiple_of(D::OutputSize::USIZE),
            "Invalid raw bytes length"
        );
        let len = bytes.len() / D::OutputSize::USIZE;
        assert!(len.is_multiple_of(2));
        let mut nodes: Vec<Output<D>> = Vec::with_capacity(len);
        for chunk in bytes.chunks_exact(D::OutputSize::USIZE) {
            let node = Output::<D>::from_slice(chunk);
            nodes.push(*node);
        }
        assert_eq!(nodes[0], Output::<D>::default());
        let len = nodes.len() / 2;
        Self {
            nodes: nodes.to_vec().into_boxed_slice(),
            len,
        }
    }
}

impl<D: Digest + FixedOutputReset> UnhashedMerkleTree<D>
where
    Output<D>: Copy,
{
    /// Finalizes the Merkle tree by hashing internal nodes
    pub fn finalize(self) -> MerkleTree<D> {
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
        MerkleTree {
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
        if self.current <= 1 {
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
    use alloy_primitives::{B256, U256};
    use alloy_sol_types::SolValue;
    use sha2::Sha256;
    use sha3::Keccak256;

    #[test]
    fn basic() {
        test_merkle_tree::<Sha256>();
        test_merkle_tree::<Keccak256>();
    }

    #[test]
    fn proof() {
        test_proof::<Sha256>();
        test_proof::<Keccak256>();
    }

    fn test_merkle_tree<D: Digest + FixedOutputReset>()
    where
        Output<D>: Copy,
    {
        let leaves = vec![
            D::digest(b"leaf1"),
            D::digest(b"leaf2"),
            D::digest(b"leaf3"),
            D::digest(b"leaf4"),
        ];

        let tree = MerkleTree::<D>::new(&leaves);

        // Manually compute the expected root
        let mut hasher = D::new();
        Digest::update(&mut hasher, [INNER_NODE_PREFIX]);
        Digest::update(&mut hasher, leaves[0]);
        Digest::update(&mut hasher, leaves[1]);
        let left_hash = hasher.finalize_reset();

        Digest::update(&mut hasher, [INNER_NODE_PREFIX]);
        Digest::update(&mut hasher, leaves[2]);
        Digest::update(&mut hasher, leaves[3]);
        let right_hash = hasher.finalize_reset();

        Digest::update(&mut hasher, [INNER_NODE_PREFIX]);
        Digest::update(&mut hasher, left_hash);
        Digest::update(&mut hasher, right_hash);
        let expected_root = hasher.finalize();

        assert_eq!(tree.root().as_slice(), expected_root.as_slice());
    }

    fn test_proof<D: Digest + FixedOutputReset>()
    where
        Output<D>: Copy,
    {
        let leaves = vec![
            D::digest(b"apple"),
            D::digest(b"banana"),
            D::digest(b"cherry"),
            D::digest(b"date"),
        ];

        let tree = MerkleTree::<D>::new(&leaves);

        for leaf in &leaves {
            let iter = tree
                .get_proof_iter(leaf)
                .expect("Leaf should be in the tree");
            let mut current_hash = *leaf;

            let mut hasher = D::new();
            for (side, sibling_hash) in iter {
                match side {
                    NodePosition::Left => {
                        Digest::update(&mut hasher, [INNER_NODE_PREFIX]);
                        Digest::update(&mut hasher, current_hash);
                        Digest::update(&mut hasher, sibling_hash);
                    }
                    NodePosition::Right => {
                        Digest::update(&mut hasher, [INNER_NODE_PREFIX]);
                        Digest::update(&mut hasher, sibling_hash);
                        Digest::update(&mut hasher, current_hash);
                    }
                }
                current_hash = hasher.finalize_reset();
            }

            assert_eq!(current_hash.as_slice(), tree.root().as_slice());
        }
    }

    #[ignore]
    #[test]
    fn generate_sol_test() {
        let mut leaves = Vec::with_capacity(1024);
        for i in 0..1024 {
            let mut hasher = Keccak256::new();
            let value = U256::from(i).abi_encode_packed();
            hasher.update(&value);
            leaves.push(hasher.finalize());
        }

        for i in 0..=10u32 {
            let tree = MerkleTree::<Keccak256>::new(&leaves[..2usize.pow(i)]);
            let root = B256::from_slice(tree.root());
            println!("bytes32({root}),");
        }
    }
}
