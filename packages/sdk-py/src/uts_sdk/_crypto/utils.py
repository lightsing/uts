# packages/sdk-py/src/uts_sdk/_crypto/utils.py
"""Cryptographic hash utilities."""

from __future__ import annotations

import hashlib
from collections.abc import Callable

try:
    from Crypto.Hash import keccak

    def keccak256(data: bytes) -> bytes:
        """Compute Keccak-256 hash (used by Ethereum).

        Note: This is NOT SHA3-256. Keccak-256 uses the original Keccak
        permutation before NIST added domain separation for SHA3.
        """
        k = keccak.new(digest_bits=256)
        k.update(data)
        return k.digest()

except ImportError:
    from web3 import Web3

    _w3 = Web3()

    def keccak256(data: bytes) -> bytes:
        """Compute Keccak-256 hash (used by Ethereum)."""
        return _w3.keccak(data)


def sha256(data: bytes) -> bytes:
    """Compute SHA-256 hash."""
    return hashlib.sha256(data).digest()


def ripemd160(data: bytes) -> bytes:
    """Compute RIPEMD-160 hash."""
    return hashlib.new("ripemd160", data).digest()


def sha1(data: bytes) -> bytes:
    """Compute SHA-1 hash (deprecated, not secure)."""
    return hashlib.sha1(data).digest()


HashFunction = Callable[[bytes], bytes]
