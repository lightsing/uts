"""Tests for encoder module."""

from __future__ import annotations

import hashlib

import pytest

from uts_sdk._codec.constants import (
    BITCOIN_TAG,
    EAS_ATTEST_TAG,
    MAGIC_BYTES,
    PENDING_TAG,
)
from uts_sdk._codec.encoder import Encoder
from uts_sdk._types import (
    AppendStep,
    AttestationStep,
    BitcoinAttestation,
    DetachedTimestamp,
    DigestHeader,
    DigestOp,
    EASAttestation,
    EASTimestamped,
    ForkStep,
    HexlifyStep,
    Keccak256Step,
    PendingAttestation,
    PrependStep,
    ReverseStep,
    RIPEMD160Step,
    SHA1Step,
    SHA256Step,
    Timestamp,
    UnknownAttestation,
)
from uts_sdk.errors import EncodeError


class TestEncoderBasics:
    def test_write_byte(self) -> None:
        encoder = Encoder()
        encoder.write_byte(0xFF)
        assert encoder.to_bytes() == b"\xff"

    def test_write_byte_out_of_range(self) -> None:
        encoder = Encoder()
        with pytest.raises(EncodeError, match="0-255"):
            encoder.write_byte(256)

    def test_write_bytes(self) -> None:
        encoder = Encoder()
        encoder.write_bytes(b"hello")
        assert encoder.to_bytes() == b"hello"

    def test_write_length_prefixed(self) -> None:
        encoder = Encoder()
        encoder.write_length_prefixed(b"test")
        assert encoder.to_bytes() == b"\x04test"

    def test_write_magic(self) -> None:
        encoder = Encoder()
        encoder.write_magic()
        data = encoder.to_bytes()
        assert data.startswith(MAGIC_BYTES)
        assert data[len(MAGIC_BYTES)] == 0x01


class TestEncoderLEB128:
    def test_u32_zero(self) -> None:
        encoder = Encoder()
        encoder.write_u32(0)
        assert encoder.to_bytes() == b"\x00"

    def test_u32_small(self) -> None:
        encoder = Encoder()
        encoder.write_u32(127)
        assert encoder.to_bytes() == b"\x7f"

    def test_u32_128(self) -> None:
        encoder = Encoder()
        encoder.write_u32(128)
        assert encoder.to_bytes() == b"\x80\x01"

    def test_u32_negative(self) -> None:
        encoder = Encoder()
        with pytest.raises(EncodeError, match="non-negative"):
            encoder.write_u32(-1)

    def test_u32_overflow(self) -> None:
        encoder = Encoder()
        with pytest.raises(EncodeError, match="exceeds maximum"):
            encoder.write_u32(0xFFFFFFFF + 1)

    def test_u64_large(self) -> None:
        encoder = Encoder()
        encoder.write_u64(2**32)
        data = encoder.to_bytes()
        assert len(data) >= 5


class TestEncoderSteps:
    def test_write_sha256_step(self) -> None:
        encoder = Encoder()
        encoder.write_step(SHA256Step())
        assert encoder.to_bytes() == b"\x08"

    def test_write_sha1_step(self) -> None:
        encoder = Encoder()
        encoder.write_step(SHA1Step())
        assert encoder.to_bytes() == b"\x02"

    def test_write_ripemd160_step(self) -> None:
        encoder = Encoder()
        encoder.write_step(RIPEMD160Step())
        assert encoder.to_bytes() == b"\x03"

    def test_write_keccak256_step(self) -> None:
        encoder = Encoder()
        encoder.write_step(Keccak256Step())
        assert encoder.to_bytes() == b"\x67"

    def test_write_reverse_step(self) -> None:
        encoder = Encoder()
        encoder.write_step(ReverseStep())
        assert encoder.to_bytes() == b"\xf2"

    def test_write_hexlify_step(self) -> None:
        encoder = Encoder()
        encoder.write_step(HexlifyStep())
        assert encoder.to_bytes() == b"\xf3"

    def test_write_append_step(self) -> None:
        encoder = Encoder()
        encoder.write_step(AppendStep(data=b"\x01\x02"))
        assert encoder.to_bytes() == b"\xf0\x02\x01\x02"

    def test_write_prepend_step(self) -> None:
        encoder = Encoder()
        encoder.write_step(PrependStep(data=b"\x01\x02"))
        assert encoder.to_bytes() == b"\xf1\x02\x01\x02"


class TestEncoderAttestations:
    def test_write_pending_attestation(self) -> None:
        encoder = Encoder()
        att = PendingAttestation(url="https://example.com")
        step = AttestationStep(attestation=att)
        encoder.write_step(step)
        data = encoder.to_bytes()
        assert data[0] == 0x00
        assert PENDING_TAG in data

    def test_write_bitcoin_attestation(self) -> None:
        encoder = Encoder()
        att = BitcoinAttestation(height=800000)
        step = AttestationStep(attestation=att)
        encoder.write_step(step)
        data = encoder.to_bytes()
        assert data[0] == 0x00
        assert BITCOIN_TAG in data

    def test_write_eas_attestation(self) -> None:
        encoder = Encoder()
        att = EASAttestation(chain_id=1, uid=b"\x00" * 32)
        step = AttestationStep(attestation=att)
        encoder.write_step(step)
        data = encoder.to_bytes()
        assert data[0] == 0x00
        assert EAS_ATTEST_TAG in data

    def test_write_eas_timestamped(self) -> None:
        encoder = Encoder()
        att = EASTimestamped(chain_id=1)
        step = AttestationStep(attestation=att)
        encoder.write_step(step)
        data = encoder.to_bytes()
        assert data[0] == 0x00

    def test_write_unknown_attestation(self) -> None:
        encoder = Encoder()
        att = UnknownAttestation(tag=b"\xff" * 8, data=b"unknown data")
        step = AttestationStep(attestation=att)
        encoder.write_step(step)
        data = encoder.to_bytes()
        assert data[0] == 0x00
        assert b"\xff" * 8 in data


class TestEncoderFork:
    def test_write_fork_step(self) -> None:
        ts1: Timestamp = [SHA256Step()]
        ts2: Timestamp = [SHA256Step()]
        fork = ForkStep(steps=[ts1, ts2])
        encoder = Encoder()
        encoder.write_step(fork)
        data = encoder.to_bytes()
        assert b"\xff" in data


class TestEncoderPendingAttestationErrors:
    def test_invalid_url_format(self) -> None:
        encoder = Encoder()
        att = PendingAttestation(url="http://example.com?query=1&foo=bar")
        with pytest.raises(EncodeError, match="Invalid URL format"):
            encoder.write_pending_attestation(att)

    def test_url_exceeds_max_length(self) -> None:
        long_url = "https://example.com/" + "a" * 1000
        with pytest.raises(ValueError, match="URL exceeds maximum"):
            PendingAttestation(url=long_url)


class TestEncodeDetached:
    def test_encode_detached_sha256(self) -> None:
        digest = hashlib.sha256(b"test").digest()
        header = DigestHeader(kind=DigestOp.SHA256, digest=digest)
        att = PendingAttestation(url="https://example.com")
        ts: Timestamp = [AttestationStep(attestation=att)]
        ots = DetachedTimestamp(header=header, timestamp=ts)

        data = Encoder.encode_detached(ots)
        assert data.startswith(MAGIC_BYTES)

    def test_encode_detached_ripemd160(self) -> None:
        digest = b"\x00" * 20
        header = DigestHeader(kind=DigestOp.RIPEMD160, digest=digest)
        ts: Timestamp = [SHA256Step()]
        ots = DetachedTimestamp(header=header, timestamp=ts)

        data = Encoder.encode_detached(ots)
        assert data.startswith(MAGIC_BYTES)
