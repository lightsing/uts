# packages/sdk-py/src/uts_sdk/errors.py
"""Error types for UTS SDK."""

from __future__ import annotations

from enum import Enum, auto
from typing import Any


class ErrorCode(Enum):
    """Error codes for UTS operations."""

    GENERAL_ERROR = auto()
    BAD_MAGIC = auto()
    UNKNOWN_OP = auto()
    INVALID_STRUCTURE = auto()
    NEGATIVE_LEB128_INPUT = auto()
    OVERFLOW = auto()
    INVALID_URI = auto()
    LENGTH_MISMATCH = auto()
    UNEXPECTED_EOF = auto()
    REMOTE_ERROR = auto()
    UNSUPPORTED_ATTESTATION = auto()
    ATTESTATION_MISMATCH = auto()


class UTSError(Exception):
    """Base exception for all UTS errors."""

    __slots__ = ("code", "offset", "context", "_message")

    def __init__(
        self,
        code: ErrorCode,
        message: str,
        *,
        offset: int | None = None,
        context: dict[str, Any] | None = None,
    ) -> None:
        super().__init__(message)
        self._message = message
        self.code = code
        self.offset = offset
        self.context = context or {}

    def __repr__(self) -> str:
        parts = [f"{self.__class__.__name__}(code={self.code.name}"]
        if self.offset is not None:
            parts.append(f", offset={self.offset}")
        parts.append(f", message={self._message!r})")
        return "".join(parts)


class EncodeError(UTSError):
    """Error during binary encoding."""

    pass


class DecodeError(UTSError):
    """Error during binary decoding."""

    pass


class RemoteError(UTSError):
    """Error from remote calendar server or RPC."""

    def __init__(
        self,
        message: str,
        context: dict[str, Any] | None = None,
    ) -> None:
        super().__init__(ErrorCode.REMOTE_ERROR, message, context=context)


class VerifyError(UTSError):
    """Error during timestamp verification."""

    pass
