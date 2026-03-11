"""Tests for Merkle tree implementation."""

from __future__ import annotations

import hashlib

import pytest

from uts_sdk._crypto.merkle import INTERNAL_PREFIX, UnorderedMerkleTree
from uts_sdk._types.status import NodePosition


def sha256(data: bytes) -> bytes:
    return hashlib.sha256(data).digest()


def test_single_leaf_tree() -> None:
    leaf = sha256(b"leaf1")
    tree = UnorderedMerkleTree.from_leaves([leaf], sha256)

    assert tree.root == leaf
    assert leaf in tree


def test_two_leaf_tree() -> None:
    a = sha256(b"a")
    b = sha256(b"b")
    tree = UnorderedMerkleTree.from_leaves([a, b], sha256)

    assert a in tree
    assert b in tree
    assert len(tree.leaves) == 2


def test_four_leaf_tree() -> None:
    leaves = [sha256(b"a"), sha256(b"b"), sha256(b"c"), sha256(b"d")]
    tree = UnorderedMerkleTree.from_leaves(leaves, sha256)

    for leaf in leaves:
        assert leaf in tree


def test_proof_for_leaf() -> None:
    leaves = [sha256(b"a"), sha256(b"b"), sha256(b"c"), sha256(b"d")]
    tree = UnorderedMerkleTree.from_leaves(leaves, sha256)

    leaf = sha256(b"a")
    proof = tree.proof_for(leaf)
    assert proof is not None
    assert len(proof) == 2


def test_verify_proof() -> None:
    leaves = [sha256(b"a"), sha256(b"b"), sha256(b"c"), sha256(b"d")]
    tree = UnorderedMerkleTree.from_leaves(leaves, sha256)

    for leaf in leaves:
        proof = tree.proof_for(leaf)
        assert proof is not None

        computed_root = leaf
        for node in proof:
            if node.position == NodePosition.LEFT:
                computed_root = sha256(INTERNAL_PREFIX + computed_root + node.sibling)
            else:
                computed_root = sha256(INTERNAL_PREFIX + node.sibling + computed_root)

        assert computed_root == tree.root, f"Proof verification failed for leaf {leaf}"


def test_proof_nonexistent_leaf() -> None:
    a = sha256(b"a")
    b = sha256(b"b")
    tree = UnorderedMerkleTree.from_leaves([a, b], sha256)

    c = sha256(b"c")
    proof = tree.proof_for(c)
    assert proof is None


def test_empty_tree_error() -> None:
    with pytest.raises(ValueError, match="at least one leaf"):
        UnorderedMerkleTree.from_leaves([], sha256)


def test_odd_leaf_count() -> None:
    leaves = [sha256(b"a"), sha256(b"b"), sha256(b"c")]
    tree = UnorderedMerkleTree.from_leaves(leaves, sha256)

    for leaf in leaves:
        assert leaf in tree

    for leaf in leaves:
        proof = tree.proof_for(leaf)
        assert proof is not None

        computed_root = leaf
        for node in proof:
            if node.position == NodePosition.LEFT:
                computed_root = sha256(INTERNAL_PREFIX + computed_root + node.sibling)
            else:
                computed_root = sha256(INTERNAL_PREFIX + node.sibling + computed_root)

        assert computed_root == tree.root


def test_serialization() -> None:
    leaves = [sha256(b"a"), sha256(b"b"), sha256(b"c"), sha256(b"d")]
    tree = UnorderedMerkleTree.from_leaves(leaves, sha256)

    serialized = bytes(tree)
    restored = UnorderedMerkleTree.from_bytes(serialized, sha256)

    assert restored.root == tree.root
    assert set(restored.leaves) == set(tree.leaves)
