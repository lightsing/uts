# packages/sdk-py/src/uts_sdk/_crypto/merkle.py
"""Unordered Merkle tree implementation."""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from typing import Callable, overload

from uts_sdk._types.status import NodePosition


@dataclass(frozen=True, slots=True)
class SiblingNode:
    """A sibling node in a Merkle proof."""

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
        return self._siblings[index]


INTERNAL_PREFIX = b"\x01"


class UnorderedMerkleTree:
    """Binary Merkle tree with flat-array, power-of-two structure.

    This implementation follows the UTS binary Merkle tree format where
    internal nodes are prefixed with 0x01 and the tree is built from leaves
    in their original order.
    """

    __slots__ = ("_leaves", "_nodes", "_root", "_hash_func")

    def __init__(
        self,
        leaves: tuple[bytes, ...],
        nodes: list[bytes],
        root: bytes,
        hash_func: Callable[[bytes], bytes],
    ) -> None:
        self._leaves = leaves
        self._nodes = nodes
        self._root = root
        self._hash_func = hash_func

    @classmethod
    def from_leaves(
        cls,
        leaves: Sequence[bytes],
        hash_func: Callable[[bytes], bytes],
    ) -> Self:
        if len(leaves) == 0:
            raise ValueError("Merkle tree must have at least one leaf")

        leaf_tuple = tuple(leaves)
        n = len(leaves)
        nodes = [hash_func(leaf) for leaf in leaves]

        level_size = n
        while level_size > 1:
            next_level = []
            for i in range(0, level_size - 1, 2):
                left = nodes[i]
                right = nodes[i + 1]
                combined = INTERNAL_PREFIX + left + right
                next_level.append(hash_func(combined))

            if level_size % 2 == 1:
                next_level.append(nodes[-1])

            nodes = next_level
            level_size = len(nodes)

        return cls(leaf_tuple, [hash_func(leaf) for leaf in leaves], nodes[0], hash_func)

    @property
    def root(self) -> bytes:
        return self._root

    @property
    def leaves(self) -> tuple[bytes, ...]:
        return self._leaves

    def __contains__(self, leaf: bytes) -> bool:
        return leaf in self._leaves

    def proof_for(self, leaf: bytes) -> MerkleProof | None:
        if leaf not in self._leaves:
            return None

        leaf_index = self._leaves.index(leaf)
        siblings: list[SiblingNode] = []

        n = len(self._leaves)
        nodes = [self._hash_func(leaf) for leaf in self._leaves]
        index = leaf_index

        level_size = n
        while level_size > 1:
            if index % 2 == 0:
                if index + 1 < level_size:
                    siblings.append(
                        SiblingNode(
                            position=NodePosition.RIGHT,
                            sibling=nodes[index + 1],
                        )
                    )
            else:
                siblings.append(
                    SiblingNode(
                        position=NodePosition.LEFT,
                        sibling=nodes[index - 1],
                    )
                )

            next_level = []
            next_indices = []
            for i in range(0, level_size - 1, 2):
                left = nodes[i]
                right = nodes[i + 1]
                combined = INTERNAL_PREFIX + left + right
                next_level.append(self._hash_func(combined))
                next_indices.append(i // 2)

            if level_size % 2 == 1:
                next_level.append(nodes[-1])
                next_indices.append(level_size // 2)

            index = index // 2
            nodes = next_level
            level_size = len(nodes)

        return MerkleProof(siblings)

    def __bytes__(self) -> bytes:
        n = len(self._leaves)
        result = bytearray()
        result.extend(n.to_bytes(4, "big"))
        for leaf in self._leaves:
            result.extend(len(leaf).to_bytes(4, "big"))
            result.extend(leaf)
        result.extend(self._root)
        return bytes(result)

    @classmethod
    def from_bytes(
        cls,
        data: bytes,
        hash_func: Callable[[bytes], bytes],
    ) -> Self:
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

        root = data[offset : offset + 32]

        return cls(tuple(leaves), [hash_func(leaf) for leaf in leaves], root, hash_func)


from typing import Self  # noqa: E402
