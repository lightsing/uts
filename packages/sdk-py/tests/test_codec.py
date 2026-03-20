"""Tests for codec round-trip."""

from __future__ import annotations

import hashlib

from uts_sdk._codec import Decoder, Encoder
from uts_sdk._types import (
    AppendStep,
    AttestationStep,
    BitcoinAttestation,
    DetachedTimestamp,
    DigestHeader,
    DigestOp,
    PendingAttestation,
    Timestamp,
)


def test_encode_decode_simple_timestamp() -> None:
    digest = hashlib.sha256(b"hello world").digest()
    header = DigestHeader(kind=DigestOp.SHA256, digest=digest)

    att = PendingAttestation(url="https://calendar.example.com")
    step = AttestationStep(attestation=att)
    timestamp: Timestamp = [step]

    ots = DetachedTimestamp(header=header, timestamp=timestamp)

    encoded = Encoder.encode_detached(ots)
    decoded = Decoder.decode_detached(encoded, strict=True)

    assert decoded.header.kind == DigestOp.SHA256
    assert decoded.header.digest == digest
    assert len(decoded.timestamp) == 1
    assert isinstance(decoded.timestamp[0], AttestationStep)
    att_step = decoded.timestamp[0]
    assert isinstance(att_step.attestation, PendingAttestation)


def test_encode_decode_append_step() -> None:
    digest = hashlib.sha256(b"test").digest()
    header = DigestHeader(kind=DigestOp.SHA256, digest=digest)

    append = AppendStep(data=b"\x00\x01\x02\x03")
    att = BitcoinAttestation(height=800000)
    step = AttestationStep(attestation=att)
    timestamp: Timestamp = [append, step]

    ots = DetachedTimestamp(header=header, timestamp=timestamp)

    encoded = Encoder.encode_detached(ots)
    decoded = Decoder.decode_detached(encoded, strict=True)

    assert len(decoded.timestamp) == 2
    assert isinstance(decoded.timestamp[0], AppendStep)
    assert decoded.timestamp[0].data == b"\x00\x01\x02\x03"


def test_leb128_encoding() -> None:
    encoder = Encoder()
    encoder.write_u32(0)
    encoder.write_u32(127)
    encoder.write_u32(128)
    encoder.write_u32(300)

    data = encoder.to_bytes()
    decoder = Decoder(data)

    assert decoder.read_u32() == 0
    assert decoder.read_u32() == 127
    assert decoder.read_u32() == 128
    assert decoder.read_u32() == 300


def test_leb64_encoding() -> None:
    encoder = Encoder()
    encoder.write_u64(0)
    encoder.write_u64(2**32)
    encoder.write_u64(2**50)

    data = encoder.to_bytes()
    decoder = Decoder(data)

    assert decoder.read_u64() == 0
    assert decoder.read_u64() == 2**32
    assert decoder.read_u64() == 2**50
