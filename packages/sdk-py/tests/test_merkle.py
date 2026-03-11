"""Tests for Merkle tree implementation."""

from __future__ import annotations

import hashlib

import pytest

from uts_sdk._crypto.merkle import MerkleProof, SiblingNode, UnorderedMerkleTree
from uts_sdk._types.status import NodePosition


def sha256(data: bytes) -> bytes:
    return hashlib.sha256(data).digest()


def test_single_leaf_tree() -> None:
    leaves = [b"leaf1"]
    tree = UnorderedMerkleTree.from_leaves(leaves, sha256)

    assert tree.root == sha256(b"leaf1")
    assert tree.leaves == (b"leaf1",)
    assert b"leaf1" in tree


def test_two_leaf_tree() -> None:
    leaves = [b"left", b"right"]
    tree = UnorderedMerkleTree.from_leaves(leaves, sha256)

    expected_root = sha256(b"\x01" + sha256(b"left") + sha256(b"right"))
    assert tree.root == expected_root


def test_four_leaf_tree() -> None:
    leaves = [b"a", b"b", b"c", b"d"]
    tree = UnorderedMerkleTree.from_leaves(leaves, sha256)

    left_subtree = sha256(b"\x01" + sha256(b"a") + sha256(b"b"))
    right_subtree = sha256(b"\x01" + sha256(b"c") + sha256(b"d"))
    expected_root = sha256(b"\x01" + left_subtree + right_subtree)

    assert tree.root == expected_root


def test_proof_for_leaf() -> None:
    leaves = [b"a", b"b", b"c", b"d"]
    tree = UnorderedMerkleTree.from_leaves(leaves, sha256)

    proof = tree.proof_for(b"a")
    assert proof is not None

    sibling_b = sha256(b"b")
    assert proof[0].sibling == sibling_b
    assert proof[0].position == NodePosition.RIGHT

    sibling_cd = sha256(b"\x01" + sha256(b"c") + sha256(b"d"))
    assert proof[1].sibling == sibling_cd
    assert proof[1].position == NodePosition.RIGHT


def test_verify_proof() -> None:
    leaves = [b"a", b"b", b"c", b"d"]
    tree = UnorderedMerkleTree.from_leaves(leaves, sha256)

    for leaf in leaves:
        proof = tree.proof_for(leaf)
        assert proof is not None

        computed_root = sha256(leaf)
        for node in proof:
            if node.position == NodePosition.LEFT:
                computed_root = sha256(b"\x01" + node.sibling + computed_root)
            else:
                computed_root = sha256(b"\x01" + computed_root + node.sibling)

        assert computed_root == tree.root


def test_proof_nonexistent_leaf() -> None:
    leaves = [b"a", b"b"]
    tree = UnorderedMerkleTree.from_leaves(leaves, sha256)

    proof = tree.proof_for(b"c")
    assert proof is None


def test_empty_tree_error() -> None:
    with pytest.raises(ValueError, match="at least one leaf"):
        UnorderedMerkleTree.from_leaves([], sha256)


def test_odd_leaf_count() -> None:
    leaves = [b"a", b"b", b"c"]
    tree = UnorderedMerkleTree.from_leaves(leaves, sha256)

    left = sha256(b"\x01" + sha256(b"a") + sha256(b"b"))
    expected_root = sha256(b"\x01" + left + sha256(b"c"))

    assert tree.root == expected_root


def test_serialization() -> None:
    leaves = [b"a", b"b", b"c", b"d"]
    tree = UnorderedMerkleTree.from_leaves(leaves, sha256)

    serialized = bytes(tree)
    restored = UnorderedMerkleTree.from_bytes(serialized, sha256)

    assert restored.root == tree.root
    assert restored.leaves == tree.leaves
