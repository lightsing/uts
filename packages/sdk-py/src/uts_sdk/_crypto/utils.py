# packages/sdk-py/src/uts_sdk/_crypto/utils.py
"""Cryptographic hash utilities."""

from __future__ import annotations

import hashlib
from typing import Callable


def sha256(data: bytes) -> bytes:
    """Compute SHA-256 hash."""
    return hashlib.sha256(data).digest()


def keccak256(data: bytes) -> bytes:
    """Compute Keccak-256 hash (used by Ethereum)."""
    return hashlib.sha3_256(data).digest()


def ripemd160(data: bytes) -> bytes:
    """Compute RIPEMD-160 hash."""
    return hashlib.new("ripemd160", data).digest()


def sha1(data: bytes) -> bytes:
    """Compute SHA-1 hash (deprecated, not secure)."""
    return hashlib.sha1(data).digest()


HashFunction = Callable[[bytes], bytes]
