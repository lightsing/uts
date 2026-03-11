# packages/sdk-py/src/uts_sdk/_types/ops.py
"""Operation codes for timestamp steps."""

from __future__ import annotations

from enum import IntEnum, unique
from typing import Final


@unique
class OpCode(IntEnum):
    """Binary operation codes for timestamp steps."""

    ATTESTATION = 0x00
    SHA1 = 0x02
    RIPEMD160 = 0x03
    SHA256 = 0x08
    KECCAK256 = 0x67
    APPEND = 0xF0
    PREPEND = 0xF1
    REVERSE = 0xF2
    HEXLIFY = 0xF3
    FORK = 0xFF


DIGEST_OPS: Final[frozenset[OpCode]] = frozenset(
    {
        OpCode.SHA256,
        OpCode.SHA1,
        OpCode.RIPEMD160,
        OpCode.KECCAK256,
    }
)

SECURE_DIGEST_OPS: Final[frozenset[OpCode]] = frozenset(
    {
        OpCode.SHA256,
        OpCode.KECCAK256,
    }
)
