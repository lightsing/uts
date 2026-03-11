# packages/sdk-py/src/uts_sdk/_codec/decoder.py
"""Binary decoder for OpenTimestamps format."""

from __future__ import annotations

import re
from typing import TYPE_CHECKING, Self

from uts_sdk._types import (
    AppendStep,
    AttestationStep,
    BitcoinAttestation,
    DetachedTimestamp,
    DigestHeader,
    DigestOp,
    EASTimestamped,
    EASAttestation,
    ForkStep,
    HexlifyStep,
    Keccak256Step,
    OpCode,
    PendingAttestation,
    PrependStep,
    RIPEMD160Step,
    ReverseStep,
    SHA1Step,
    SHA256Step,
    Step,
    Timestamp,
    UnknownAttestation,
)
from uts_sdk.errors import DecodeError, ErrorCode

from .constants import (
    BITCOIN_TAG,
    EAS_ATTEST_TAG,
    EAS_TIMESTAMPED_TAG,
    MAGIC_BYTES,
    MAX_URI_LEN,
    PENDING_TAG,
    TAG_SIZE,
)

if TYPE_CHECKING:
    pass

_SAFE_URL_RE = re.compile(r"^https?://[\s\w./:-]+$")


def _op_from_code(code: int) -> OpCode | None:
    """Convert opcode byte to OpCode enum."""
    try:
        return OpCode(code)
    except ValueError:
        return None


class Decoder:
    """Binary decoder for OpenTimestamps proof format.

    Example:
        decoder = Decoder(data)
        version = decoder.read_magic()
        header = decoder.read_header()
        timestamp = decoder.read_timestamp()
    """

    __slots__ = ("_data", "_offset", "_length")

    def __init__(self, data: bytes) -> None:
        self._data = data
        self._offset = 0
        self._length = len(data)

    @property
    def remaining(self) -> int:
        """Number of bytes remaining to read."""
        return self._length - self._offset

    def _check_bounds(self, required: int) -> None:
        if self._offset + required > self._length:
            raise DecodeError(
                ErrorCode.UNEXPECTED_EOF,
                f"Unexpected end of stream: needed {required} bytes but only {self.remaining} available",
                offset=self._offset,
            )

    def check_eof(self) -> None:
        """Verify all bytes have been consumed."""
        if self.remaining > 0:
            raise DecodeError(
                ErrorCode.INVALID_STRUCTURE,
                f"Expected end of stream but {self.remaining} bytes remain",
                offset=self._offset,
            )

    def read_byte(self) -> int:
        """Read a single byte."""
        self._check_bounds(1)
        value = self._data[self._offset]
        self._offset += 1
        return value

    def read_bytes(self, length: int) -> bytes:
        """Read a fixed number of bytes."""
        self._check_bounds(length)
        result = self._data[self._offset : self._offset + length]
        self._offset += length
        return result

    def read_u32(self) -> int:
        """Read unsigned 32-bit integer using LEB128 decoding."""
        result = 0
        shift = 0

        while True:
            byte = self.read_byte()
            result |= (byte & 0x7F) << shift

            if (byte & 0x80) == 0:
                break

            shift += 7
            if shift > 35:
                raise DecodeError(
                    ErrorCode.OVERFLOW,
                    "LEB128 varint too long for u32",
                    offset=self._offset,
                )

        if result > 0xFFFFFFFF:
            raise DecodeError(
                ErrorCode.OVERFLOW,
                f"Value exceeds maximum for u32: {result}",
                offset=self._offset,
            )

        return result

    def read_u64(self) -> int:
        """Read unsigned 64-bit integer using LEB128 decoding."""
        result = 0
        shift = 0

        while True:
            byte = self.read_byte()
            result |= (byte & 0x7F) << shift

            if (byte & 0x80) == 0:
                break

            shift += 7
            if shift > 70:
                raise DecodeError(
                    ErrorCode.OVERFLOW,
                    "LEB128 varint too long for u64",
                    offset=self._offset,
                )

        return result

    def read_length_prefixed(self) -> bytes:
        """Read length-prefixed bytes."""
        length = self.read_u32()
        return self.read_bytes(length)

    def peek_op(self) -> OpCode | None:
        """Peek at the next opcode without consuming it."""
        if self.remaining == 0:
            return None
        return _op_from_code(self._data[self._offset])

    def read_op(self) -> OpCode:
        """Read and return an opcode."""
        code = self.read_byte()
        op = _op_from_code(code)
        if op is None:
            raise DecodeError(
                ErrorCode.UNKNOWN_OP,
                f"Unknown opcode: 0x{code:02x}",
                offset=self._offset - 1,
            )
        return op

    def read_magic(self) -> int:
        """Read and verify magic bytes, return version."""
        magic = self.read_bytes(len(MAGIC_BYTES))
        if magic != MAGIC_BYTES:
            raise DecodeError(
                ErrorCode.BAD_MAGIC,
                f"Invalid magic bytes: expected {MAGIC_BYTES.hex()}, got {magic.hex()}",
                offset=self._offset - len(MAGIC_BYTES),
            )
        return self.read_byte()

    def read_header(self) -> DigestHeader:
        """Read a digest header."""
        op = self.read_op()

        if op not in (OpCode.SHA256, OpCode.SHA1, OpCode.RIPEMD160, OpCode.KECCAK256):
            raise DecodeError(
                ErrorCode.INVALID_STRUCTURE,
                f"Expected digest op in header, got: {op.name}",
                offset=self._offset - 1,
            )

        kind = {
            OpCode.SHA256: DigestOp.SHA256,
            OpCode.SHA1: DigestOp.SHA1,
            OpCode.RIPEMD160: DigestOp.RIPEMD160,
            OpCode.KECCAK256: DigestOp.KECCAK256,
        }[op]

        lengths = {
            DigestOp.SHA256: 32,
            DigestOp.SHA1: 20,
            DigestOp.RIPEMD160: 20,
            DigestOp.KECCAK256: 32,
        }

        digest = self.read_bytes(lengths[kind])
        return DigestHeader(kind=kind, digest=digest)

    def read_execution_step(
        self,
    ) -> (
        AppendStep
        | PrependStep
        | SHA256Step
        | SHA1Step
        | RIPEMD160Step
        | Keccak256Step
        | ReverseStep
        | HexlifyStep
    ):
        """Read an execution step."""
        op = self.read_op()

        match op:
            case OpCode.APPEND:
                data = self.read_length_prefixed()
                return AppendStep(data=data)
            case OpCode.PREPEND:
                data = self.read_length_prefixed()
                return PrependStep(data=data)
            case OpCode.SHA256:
                return SHA256Step()
            case OpCode.SHA1:
                return SHA1Step()
            case OpCode.RIPEMD160:
                return RIPEMD160Step()
            case OpCode.KECCAK256:
                return Keccak256Step()
            case OpCode.REVERSE:
                return ReverseStep()
            case OpCode.HEXLIFY:
                return HexlifyStep()
            case OpCode.FORK | OpCode.ATTESTATION:
                raise DecodeError(
                    ErrorCode.INVALID_STRUCTURE,
                    f"Unexpected {op.name} step in execution step",
                    offset=self._offset - 1,
                )

    def read_fork_step(self) -> ForkStep:
        """Read a FORK step."""
        steps: list[Timestamp] = []

        if self.peek_op() != OpCode.FORK:
            raise DecodeError(
                ErrorCode.INVALID_STRUCTURE,
                f"Expected FORK op at the beginning of fork step, got: {self.peek_op()}",
                offset=self._offset,
            )

        while True:
            match self.peek_op():
                case OpCode.FORK:
                    self.read_op()
                    steps.append(self.read_timestamp())
                case _:
                    steps.append(self.read_timestamp())
                    break

        return ForkStep(steps=steps)

    def read_pending_attestation(self) -> PendingAttestation:
        """Read a pending attestation."""
        url_bytes = self.read_length_prefixed()

        try:
            url = url_bytes.decode("utf-8")
        except UnicodeDecodeError as e:
            raise DecodeError(
                ErrorCode.INVALID_URI,
                f"Invalid UTF-8 in URL: {e}",
                offset=self._offset,
            )

        if len(url) > MAX_URI_LEN:
            raise DecodeError(
                ErrorCode.INVALID_URI,
                f"URL exceeds maximum length of {MAX_URI_LEN}: {len(url)}",
                offset=self._offset,
            )

        if not _SAFE_URL_RE.match(url):
            raise DecodeError(
                ErrorCode.INVALID_URI,
                f"Invalid URL format: {url}",
                offset=self._offset,
            )

        return PendingAttestation(url=url)

    def read_bitcoin_attestation(self) -> BitcoinAttestation:
        """Read a Bitcoin attestation."""
        height = self.read_u32()
        return BitcoinAttestation(height=height)

    def read_eas_attestation(self) -> EASAttestation:
        """Read an EAS attestation."""
        chain_id = self.read_u64()
        uid = self.read_bytes(32)
        return EASAttestation(chain_id=chain_id, uid=uid)

    def read_eas_timestamped(self) -> EASTimestamped:
        """Read an EAS timestamped attestation."""
        chain_id = self.read_u64()
        return EASTimestamped(chain_id=chain_id)

    def read_attestation_step(self, *, strict: bool = False) -> AttestationStep:
        """Read an attestation step."""
        op = self.read_op()

        if op != OpCode.ATTESTATION:
            raise DecodeError(
                ErrorCode.INVALID_STRUCTURE,
                f"Expected ATTESTATION op, got: {op.name}",
                offset=self._offset - 1,
            )

        tag = self.read_bytes(TAG_SIZE)
        data = self.read_length_prefixed()

        att: (
            PendingAttestation
            | BitcoinAttestation
            | EASAttestation
            | EASTimestamped
            | UnknownAttestation
        )

        match tag:
            case bytes(t) if t == PENDING_TAG:
                inner = Decoder(data)
                att = inner.read_pending_attestation()
                if strict:
                    inner.check_eof()
            case bytes(t) if t == BITCOIN_TAG:
                inner = Decoder(data)
                att = inner.read_bitcoin_attestation()
                if strict:
                    inner.check_eof()
            case bytes(t) if t == EAS_ATTEST_TAG:
                inner = Decoder(data)
                att = inner.read_eas_attestation()
                if strict:
                    inner.check_eof()
            case bytes(t) if t == EAS_TIMESTAMPED_TAG:
                inner = Decoder(data)
                att = inner.read_eas_timestamped()
                if strict:
                    inner.check_eof()
            case _:
                att = UnknownAttestation(tag=tag, data=data)

        return AttestationStep(attestation=att)

    def read_step(self, *, strict: bool = False) -> Step:
        """Read any step type."""
        match self.peek_op():
            case OpCode.FORK:
                return self.read_fork_step()
            case OpCode.ATTESTATION:
                return self.read_attestation_step(strict=strict)
            case _:
                return self.read_execution_step()

    def read_timestamp(self, *, strict: bool = False) -> Timestamp:
        """Read a timestamp (sequence of steps ending with FORK or ATTESTATION)."""
        steps: list[Step] = []

        while self.remaining > 0:
            step = self.read_step(strict=strict)
            steps.append(step)

            match step:
                case ForkStep() | AttestationStep():
                    break

        return steps

    @classmethod
    def decode_detached(cls, data: bytes, *, strict: bool = False) -> DetachedTimestamp:
        """Decode a complete detached timestamp file."""
        decoder = cls(data)

        version = decoder.read_magic()
        if version != 0x01:
            raise DecodeError(
                ErrorCode.INVALID_STRUCTURE,
                f"Unsupported detached timestamp version: 0x{version:02x}",
                offset=decoder._offset - 1,
            )

        header = decoder.read_header()
        timestamp = decoder.read_timestamp(strict=strict)

        if strict:
            decoder.check_eof()

        return DetachedTimestamp(header=header, timestamp=timestamp)
