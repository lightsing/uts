# packages/sdk-py/src/uts_sdk/_codec/encoder.py
"""Binary encoder for OpenTimestamps format."""

from __future__ import annotations

import re
from typing import Self

from uts_sdk._types import (
    AppendStep,
    AttestationStep,
    BitcoinAttestation,
    EASTimestamped,
    EASAttestation,
    ExecutionStep,
    ForkStep,
    PendingAttestation,
    PrependStep,
    UnknownAttestation,
)
from uts_sdk.errors import EncodeError, ErrorCode

from .constants import (
    BITCOIN_TAG,
    EAS_ATTEST_TAG,
    EAS_TIMESTAMPED_TAG,
    MAGIC_BYTES,
    MAX_URI_LEN,
    PENDING_TAG,
)

_SAFE_URL_RE = re.compile(r"^https?://[\s\w./:-]+$")


class Encoder:
    """Binary encoder for OpenTimestamps proof format.

    Example:
        encoder = Encoder()
        encoder.write_u32(42).write_bytes(b"hello")
        data = encoder.to_bytes()
    """

    __slots__ = ("_buffer", "_offset")

    def __init__(self, initial_size: int = 1024) -> None:
        self._buffer = bytearray(initial_size)
        self._offset = 0

    def _ensure_capacity(self, required: int) -> None:
        if self._offset + required > len(self._buffer):
            new_size = max(len(self._buffer) * 2, self._offset + required)
            self._buffer.extend(b"\x00" * (new_size - len(self._buffer)))

    def to_bytes(self) -> bytes:
        return bytes(self._buffer[: self._offset])

    def write_byte(self, value: int) -> Self:
        if not 0 <= value <= 255:
            raise EncodeError(
                ErrorCode.OVERFLOW,
                f"Byte value must be 0-255, got {value}",
            )
        self._ensure_capacity(1)
        self._buffer[self._offset] = value
        self._offset += 1
        return self

    def write_bytes(self, data: bytes) -> Self:
        self._ensure_capacity(len(data))
        self._buffer[self._offset : self._offset + len(data)] = data
        self._offset += len(data)
        return self

    def write_u32(self, value: int) -> Self:
        if value < 0:
            raise EncodeError(
                ErrorCode.NEGATIVE_LEB128_INPUT,
                f"LEB128 only supports non-negative integers, got {value}",
            )
        if value > 0xFFFFFFFF:
            raise EncodeError(
                ErrorCode.OVERFLOW,
                f"Value exceeds maximum for u32: {value}",
            )

        remaining = value
        while True:
            byte = remaining & 0x7F
            remaining >>= 7
            if remaining != 0:
                byte |= 0x80
            self.write_byte(byte)
            if remaining == 0:
                break
        return self

    def write_u64(self, value: int) -> Self:
        if value < 0:
            raise EncodeError(
                ErrorCode.NEGATIVE_LEB128_INPUT,
                f"LEB128 only supports non-negative integers, got {value}",
            )

        remaining = value
        while True:
            byte = remaining & 0x7F
            remaining >>= 7
            if remaining != 0:
                byte |= 0x80
            self.write_byte(byte)
            if remaining == 0:
                break
        return self

    def write_length_prefixed(self, data: bytes) -> Self:
        self.write_u32(len(data))
        self.write_bytes(data)
        return self

    def write_magic(self, version: int = 0x01) -> Self:
        self.write_bytes(MAGIC_BYTES)
        self.write_byte(version)
        return self

    def write_header(self, header) -> Self:
        self.write_byte(header.kind.to_op_code().value)
        self.write_bytes(header.digest)
        return self

    def write_execution_step(self, step: ExecutionStep) -> Self:
        self.write_byte(step.op.value)
        if isinstance(step, (AppendStep, PrependStep)):
            self.write_length_prefixed(step.data)
        return self

    def write_fork_step(self, step: ForkStep) -> Self:
        for branch in step.steps[:-1]:
            self.write_byte(0xFF)
            self.write_timestamp(branch)
        self.write_timestamp(step.steps[-1])
        return self

    def write_pending_attestation(self, att: PendingAttestation) -> Self:
        url = att.url.rstrip("/")
        if len(url) > MAX_URI_LEN:
            raise EncodeError(
                ErrorCode.INVALID_URI,
                f"URL exceeds maximum length of {MAX_URI_LEN}: {len(url)}",
            )
        if not _SAFE_URL_RE.match(url):
            raise EncodeError(
                ErrorCode.INVALID_URI,
                f"Invalid URL format: {url}",
            )
        self.write_length_prefixed(url.encode("utf-8"))
        return self

    def write_bitcoin_attestation(self, att: BitcoinAttestation) -> Self:
        self.write_u32(att.height)
        return self

    def write_eas_attestation(self, att: EASAttestation | EASTimestamped) -> Self:
        self.write_u64(att.chain_id)
        if isinstance(att, EASAttestation):
            self.write_bytes(att.uid)
        return self

    def write_attestation_step(self, step: AttestationStep) -> Self:
        self.write_byte(0x00)

        att = step.attestation
        if isinstance(att, PendingAttestation):
            self.write_bytes(PENDING_TAG)
            inner = Encoder()
            inner.write_pending_attestation(att)
            self.write_length_prefixed(inner.to_bytes())
        elif isinstance(att, BitcoinAttestation):
            self.write_bytes(BITCOIN_TAG)
            inner = Encoder()
            inner.write_bitcoin_attestation(att)
            self.write_length_prefixed(inner.to_bytes())
        elif isinstance(att, EASAttestation):
            self.write_bytes(EAS_ATTEST_TAG)
            inner = Encoder()
            inner.write_eas_attestation(att)
            self.write_length_prefixed(inner.to_bytes())
        elif isinstance(att, EASTimestamped):
            self.write_bytes(EAS_TIMESTAMPED_TAG)
            inner = Encoder()
            inner.write_eas_attestation(att)
            self.write_length_prefixed(inner.to_bytes())
        elif isinstance(att, UnknownAttestation):
            self.write_bytes(att.tag)
            self.write_length_prefixed(att.data)
        else:
            raise EncodeError(
                ErrorCode.GENERAL_ERROR,
                f"Unsupported attestation type: {type(att).__name__}",
            )
        return self

    def write_step(self, step) -> Self:
        if isinstance(step, ForkStep):
            return self.write_fork_step(step)
        elif isinstance(step, AttestationStep):
            return self.write_attestation_step(step)
        else:
            return self.write_execution_step(step)

    def write_timestamp(self, timestamp) -> Self:
        for step in timestamp:
            self.write_step(step)
        return self

    @classmethod
    def encode_detached(cls, ots) -> bytes:
        encoder = cls()
        encoder.write_magic()
        encoder.write_header(ots.header)
        encoder.write_timestamp(ots.timestamp)
        return encoder.to_bytes()
