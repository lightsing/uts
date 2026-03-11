# packages/sdk-py/src/uts_sdk/_types/digest.py
"""Digest types for timestamp headers."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import TYPE_CHECKING, Literal

from .ops import OpCode

if TYPE_CHECKING:
    from .timestamp_steps import Timestamp


class DigestOp(str, Enum):
    """Supported digest operations."""

    SHA256 = "SHA256"
    SHA1 = "SHA1"
    RIPEMD160 = "RIPEMD160"
    KECCAK256 = "KECCAK256"

    def to_op_code(self) -> OpCode:
        """Convert to binary operation code."""
        mapping = {
            DigestOp.SHA256: OpCode.SHA256,
            DigestOp.SHA1: OpCode.SHA1,
            DigestOp.RIPEMD160: OpCode.RIPEMD160,
            DigestOp.KECCAK256: OpCode.KECCAK256,
        }
        return mapping[self]


SecureDigestOp = Literal[DigestOp.SHA256, DigestOp.KECCAK256]


@dataclass(frozen=True, slots=True, kw_only=True)
class DigestHeader:
    """Header containing digest algorithm and value."""

    digest: bytes
    kind: DigestOp

    def __post_init__(self) -> None:
        expected_lengths = {
            DigestOp.SHA256: 32,
            DigestOp.SHA1: 20,
            DigestOp.RIPEMD160: 20,
            DigestOp.KECCAK256: 32,
        }
        expected = expected_lengths[self.kind]
        if len(self.digest) != expected:
            raise ValueError(
                f"Digest length mismatch for {self.kind.value}: "
                f"expected {expected}, got {len(self.digest)}"
            )


@dataclass(frozen=True, slots=True, kw_only=True)
class DetachedTimestamp:
    """A detached timestamp file containing digest header and proof steps."""

    header: DigestHeader
    timestamp: Timestamp
