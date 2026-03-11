"""Tests for crypto utilities."""

from __future__ import annotations

import hashlib

from uts_sdk._crypto.utils import sha256, keccak256, sha1, ripemd160, HashFunction


class TestHashFunctions:
    def test_sha256(self) -> None:
        result = sha256(b"hello")
        expected = hashlib.sha256(b"hello").digest()
        assert result == expected
        assert len(result) == 32

    def test_keccak256(self) -> None:
        result = keccak256(b"hello")
        expected = hashlib.sha3_256(b"hello").digest()
        assert result == expected
        assert len(result) == 32

    def test_sha1(self) -> None:
        result = sha1(b"hello")
        expected = hashlib.sha1(b"hello").digest()
        assert result == expected
        assert len(result) == 20

    def test_ripemd160(self) -> None:
        result = ripemd160(b"hello")
        expected = hashlib.new("ripemd160", b"hello").digest()
        assert result == expected
        assert len(result) == 20

    def test_hash_function_protocol(self) -> None:
        def custom_hash(data: bytes) -> bytes:
            return data + b"\x00"

        h: HashFunction = custom_hash
        assert h(b"test") == b"test\x00"
