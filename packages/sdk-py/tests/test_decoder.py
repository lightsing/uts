"""Tests for decoder module."""

from __future__ import annotations

import hashlib
import pytest

from uts_sdk._codec.decoder import Decoder
from uts_sdk._codec.constants import MAGIC_BYTES
from uts_sdk._types import (
    AppendStep,
    AttestationStep,
    BitcoinAttestation,
    DigestOp,
    EASTimestamped,
    EASAttestation,
    ForkStep,
    PendingAttestation,
    PrependStep,
    RIPEMD160Step,
    SHA1Step,
    SHA256Step,
    Keccak256Step,
    ReverseStep,
    HexlifyStep,
    UnknownAttestation,
)
from uts_sdk.errors import DecodeError, ErrorCode


class TestDecoderBasics:
    def test_read_byte(self) -> None:
        decoder = Decoder(b"\xff\x00")
        assert decoder.read_byte() == 0xFF
        assert decoder.read_byte() == 0x00

    def test_read_bytes(self) -> None:
        decoder = Decoder(b"hello")
        assert decoder.read_bytes(5) == b"hello"

    def test_remaining(self) -> None:
        decoder = Decoder(b"\x00\x01\x02")
        assert decoder.remaining == 3
        decoder.read_byte()
        assert decoder.remaining == 2

    def test_check_eof_success(self) -> None:
        decoder = Decoder(b"\x00")
        decoder.read_byte()
        decoder.check_eof()

    def test_check_eof_failure(self) -> None:
        decoder = Decoder(b"\x00\x01")
        decoder.read_byte()
        with pytest.raises(DecodeError, match="bytes remain"):
            decoder.check_eof()


class TestDecoderLEB128:
    def test_u32_zero(self) -> None:
        decoder = Decoder(b"\x00")
        assert decoder.read_u32() == 0

    def test_u32_small(self) -> None:
        decoder = Decoder(b"\x7f")
        assert decoder.read_u32() == 127

    def test_u32_128(self) -> None:
        decoder = Decoder(b"\x80\x01")
        assert decoder.read_u32() == 128

    def test_u32_300(self) -> None:
        decoder = Decoder(b"\xac\x02")
        assert decoder.read_u32() == 300

    def test_u64_large(self) -> None:
        decoder = Decoder(b"\x80\x80\x80\x80\x10")
        assert decoder.read_u64() == 2**32


class TestDecoderMagic:
    def test_valid_magic(self) -> None:
        data = MAGIC_BYTES + b"\x01"
        decoder = Decoder(data)
        version = decoder.read_magic()
        assert version == 0x01

    def test_invalid_magic(self) -> None:
        invalid_magic = b"\x00" * len(MAGIC_BYTES)
        decoder = Decoder(invalid_magic)
        with pytest.raises(DecodeError, match="Invalid magic"):
            decoder.read_magic()


class TestDecoderHeader:
    def test_sha256_header(self) -> None:
        digest = hashlib.sha256(b"test").digest()
        data = b"\x08" + digest
        decoder = Decoder(data)
        header = decoder.read_header()
        assert header.kind == DigestOp.SHA256
        assert header.digest == digest

    def test_sha1_header(self) -> None:
        digest = hashlib.sha1(b"test").digest()
        data = b"\x02" + digest
        decoder = Decoder(data)
        header = decoder.read_header()
        assert header.kind == DigestOp.SHA1

    def test_ripemd160_header(self) -> None:
        import hashlib

        digest = hashlib.new("ripemd160", b"test").digest()
        data = b"\x03" + digest
        decoder = Decoder(data)
        header = decoder.read_header()
        assert header.kind == DigestOp.RIPEMD160

    def test_invalid_header_op(self) -> None:
        decoder = Decoder(b"\x00" + b"\x00" * 32)
        with pytest.raises(DecodeError, match="Expected digest op"):
            decoder.read_header()


class TestDecoderExecutionSteps:
    def test_append_step(self) -> None:
        decoder = Decoder(b"\xf0\x03\x01\x02\x03")
        step = decoder.read_execution_step()
        assert isinstance(step, AppendStep)
        assert step.data == b"\x01\x02\x03"

    def test_prepend_step(self) -> None:
        decoder = Decoder(b"\xf1\x02AB")
        step = decoder.read_execution_step()
        assert isinstance(step, PrependStep)
        assert step.data == b"AB"

    def test_sha256_step(self) -> None:
        decoder = Decoder(b"\x08")
        step = decoder.read_execution_step()
        assert isinstance(step, SHA256Step)

    def test_sha1_step(self) -> None:
        decoder = Decoder(b"\x02")
        step = decoder.read_execution_step()
        assert isinstance(step, SHA1Step)

    def test_ripemd160_step(self) -> None:
        decoder = Decoder(b"\x03")
        step = decoder.read_execution_step()
        assert isinstance(step, RIPEMD160Step)

    def test_keccak256_step(self) -> None:
        decoder = Decoder(b"\x67")
        step = decoder.read_execution_step()
        assert isinstance(step, Keccak256Step)

    def test_reverse_step(self) -> None:
        decoder = Decoder(b"\xf2")
        step = decoder.read_execution_step()
        assert isinstance(step, ReverseStep)

    def test_hexlify_step(self) -> None:
        decoder = Decoder(b"\xf3")
        step = decoder.read_execution_step()
        assert isinstance(step, HexlifyStep)


class TestDecoderAttestations:
    def test_unknown_attestation(self) -> None:
        unknown_tag = b"\xff" * 8
        data = b"\x00" + unknown_tag + b"\x04test"
        decoder = Decoder(data)
        step = decoder.read_attestation_step()
        assert isinstance(step, AttestationStep)
        assert isinstance(step.attestation, UnknownAttestation)
        assert step.attestation.tag == unknown_tag


class TestDecoderErrors:
    def test_unexpected_eof(self) -> None:
        decoder = Decoder(b"\x01")
        with pytest.raises(DecodeError, match="Unexpected end"):
            decoder.read_bytes(10)

    def test_unknown_opcode(self) -> None:
        decoder = Decoder(b"\x99")
        with pytest.raises(DecodeError, match="Unknown opcode"):
            decoder.read_op()


class TestDecoderLengthPrefixed:
    def test_read_length_prefixed(self) -> None:
        decoder = Decoder(b"\x05hello")
        data = decoder.read_length_prefixed()
        assert data == b"hello"

    def test_read_length_prefixed_empty(self) -> None:
        decoder = Decoder(b"\x00")
        data = decoder.read_length_prefixed()
        assert data == b""
