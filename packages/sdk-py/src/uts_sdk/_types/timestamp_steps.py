# packages/sdk-py/src/uts_sdk/_types/timestamp_steps.py
"""Timestamp step types for building timestamp proofs."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Literal

from .attestations import Attestation
from .ops import OpCode

if TYPE_CHECKING:
    pass


@dataclass(frozen=True, slots=True, kw_only=True)
class AppendStep:
    """Append data to the current digest."""

    data: bytes
    op: Literal[OpCode.APPEND] = OpCode.APPEND


@dataclass(frozen=True, slots=True, kw_only=True)
class PrependStep:
    """Prepend data to the current digest."""

    data: bytes
    op: Literal[OpCode.PREPEND] = OpCode.PREPEND


@dataclass(frozen=True, slots=True)
class SHA256Step:
    """SHA-256 hash operation."""

    op: Literal[OpCode.SHA256] = OpCode.SHA256


@dataclass(frozen=True, slots=True)
class SHA1Step:
    """SHA-1 hash operation (deprecated, not secure)."""

    op: Literal[OpCode.SHA1] = OpCode.SHA1


@dataclass(frozen=True, slots=True)
class RIPEMD160Step:
    """RIPEMD-160 hash operation."""

    op: Literal[OpCode.RIPEMD160] = OpCode.RIPEMD160


@dataclass(frozen=True, slots=True)
class Keccak256Step:
    """Keccak-256 hash operation."""

    op: Literal[OpCode.KECCAK256] = OpCode.KECCAK256


@dataclass(frozen=True, slots=True)
class ReverseStep:
    """Reverse the byte order of the digest."""

    op: Literal[OpCode.REVERSE] = OpCode.REVERSE


@dataclass(frozen=True, slots=True)
class HexlifyStep:
    """Convert digest to hexadecimal string."""

    op: Literal[OpCode.HEXLIFY] = OpCode.HEXLIFY


@dataclass(frozen=True, slots=True, kw_only=True)
class AttestationStep:
    """Attestation step containing proof from a time-stamping authority."""

    attestation: Attestation
    op: Literal[OpCode.ATTESTATION] = OpCode.ATTESTATION


@dataclass(frozen=True, slots=True)
class ForkStep:
    """Fork into multiple parallel proof paths."""

    steps: list[Timestamp]
    op: Literal[OpCode.FORK] = OpCode.FORK

    def __post_init__(self) -> None:
        if len(self.steps) < 2:
            raise ValueError(
                f"FORK step must have at least 2 branches, got {len(self.steps)}"
            )


DigestOp = SHA256Step | SHA1Step | RIPEMD160Step | Keccak256Step
DataStep = AppendStep | PrependStep
UnaryOp = DigestOp | ReverseStep | HexlifyStep
ExecutionStep = DataStep | UnaryOp
Step = ExecutionStep | AttestationStep | ForkStep
Timestamp = list[Step]
