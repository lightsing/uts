# packages/sdk-py/src/uts_sdk/_crypto/merkle.py
"""Unordered Merkle tree implementation.

This follows the UTS binary Merkle tree format where:
- Internal nodes are prefixed with 0x01
- Leaves are sorted lexicographically
- The tree is built as a flat array structure for efficient proof generation
"""

from __future__ import annotations

from collections.abc import Callable, Sequence
from dataclasses import dataclass
from typing import overload

from typing_extensions import Self

from uts_sdk._types.status import NodePosition


@dataclass(frozen=True, slots=True)
class SiblingNode:
    """A sibling node in a Merkle proof.

    Attributes:
        position: The position indicates how to combine with sibling:
            - LEFT: sibling is right child, APPEND sibling when computing parent
            - RIGHT: sibling is left child, PREPEND sibling when computing parent
        sibling: The sibling hash bytes
    """

    position: NodePosition
    sibling: bytes


class MerkleProof(Sequence[SiblingNode]):
    """A Merkle proof as a sequence of sibling nodes."""

    __slots__ = ("_siblings",)

    def __init__(self, siblings: list[SiblingNode]) -> None:
        self._siblings = siblings

    def __len__(self) -> int:
        return len(self._siblings)

    @overload
    def __getitem__(self, index: int) -> SiblingNode: ...

    @overload
    def __getitem__(self, index: slice) -> Sequence[SiblingNode]: ...

    def __getitem__(self, index: int | slice) -> SiblingNode | Sequence[SiblingNode]:
        return self._siblings[index]


INTERNAL_PREFIX = b"\x01"


def _next_power_of_two(n: int) -> int:
    """Return the next power of two >= n."""
    if n <= 1:
        return 1
    p = 1
    while p < n:
        p *= 2
    return p


def _compare_bytes(a: bytes, b: bytes) -> int:
    """Compare two byte strings lexicographically."""
    min_len = min(len(a), len(b))
    for i in range(min_len):
        if a[i] != b[i]:
            return a[i] - b[i]
    return len(a) - len(b)


class UnorderedMerkleTree:
    """Flat, fixed-size Merkle tree with sorted leaves.

    This implementation uses a flat array structure where:
    - Index 0 is unused
    - Leaves start at index `len` (the tree size, which is a power of two)
    - Internal nodes fill indices 1 to len-1

    This allows O(log n) proof generation without recomputing the tree.
    """

    __slots__ = ("_nodes", "_len", "_leaf_indices", "_hash_func")

    def __init__(
        self,
        nodes: list[bytes],
        tree_len: int,
        leaf_indices: dict[bytes, int],
        hash_func: Callable[[bytes], bytes],
    ) -> None:
        self._nodes = nodes
        self._len = tree_len
        self._leaf_indices = leaf_indices
        self._hash_func = hash_func

    @classmethod
    def from_leaves(
        cls,
        leaves: Sequence[bytes],
        hash_func: Callable[[bytes], bytes],
    ) -> Self:
        """Build a Merkle tree from leaves."""
        if len(leaves) == 0:
            raise ValueError("Merkle tree must have at least one leaf")

        raw_len = len(leaves)
        tree_len = _next_power_of_two(raw_len)
        nodes = [b"\x00" * 32] * (2 * tree_len)  # Index 0 unused, so 2*len

        # Hash and sort leaves
        hashed_leaves = [(hash_func(leaf), leaf) for leaf in leaves]
        hashed_leaves.sort(key=lambda x: x[0])

        # Map original leaf -> index in sorted hashed leaves
        leaf_indices: dict[bytes, int] = {}
        for i, (hashed, orig_leaf) in enumerate(hashed_leaves):
            nodes[tree_len + i] = hashed
            leaf_indices[orig_leaf] = i

        # Pad remaining leaf slots with zeros
        for i in range(raw_len, tree_len):
            nodes[tree_len + i] = b"\x00" * 32

        # Build internal nodes (from bottom to top)
        for i in range(tree_len - 1, 0, -1):
            left = nodes[2 * i]
            right = nodes[2 * i + 1]
            combined = INTERNAL_PREFIX + left + right
            nodes[i] = hash_func(combined)

        return cls(nodes, tree_len, leaf_indices, hash_func)

    @property
    def root(self) -> bytes:
        return self._nodes[1]

    @property
    def leaves(self) -> tuple[bytes, ...]:
        return tuple(self._leaf_indices.keys())

    def __contains__(self, leaf: bytes) -> bool:
        return leaf in self._leaf_indices

    def proof_for(self, leaf: bytes) -> MerkleProof | None:
        """Generate a Merkle proof for the given leaf.

        Returns None if the leaf is not in the tree.
        """
        if leaf not in self._leaf_indices:
            return None

        siblings: list[SiblingNode] = []
        current = self._len + self._leaf_indices[leaf]

        while current > 1:
            is_left = (current & 1) == 0
            # When current is left child (even), position is LEFT (sibling is right)
            # When current is right child (odd), position is RIGHT (sibling is left)
            position = NodePosition.LEFT if is_left else NodePosition.RIGHT
            sibling_index = current ^ 1  # XOR to get sibling
            siblings.append(
                SiblingNode(position=position, sibling=self._nodes[sibling_index])
            )
            current >>= 1

        return MerkleProof(siblings)

    def __bytes__(self) -> bytes:
        result = bytearray()
        result.extend(len(self._leaf_indices).to_bytes(4, "big"))
        for leaf in self._leaf_indices:
            result.extend(len(leaf).to_bytes(4, "big"))
            result.extend(leaf)
        result.extend(self.root)
        return bytes(result)

    @classmethod
    def from_bytes(
        cls,
        data: bytes,
        hash_func: Callable[[bytes], bytes],
    ) -> Self:
        """Reconstruct tree from serialized bytes."""
        offset = 0
        n = int.from_bytes(data[offset : offset + 4], "big")
        offset += 4

        leaves = []
        for _ in range(n):
            leaf_len = int.from_bytes(data[offset : offset + 4], "big")
            offset += 4
            leaf = data[offset : offset + leaf_len]
            offset += leaf_len
            leaves.append(leaf)

        return cls.from_leaves(leaves, hash_func)
