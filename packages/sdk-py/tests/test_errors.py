"""Tests for error classes."""

from __future__ import annotations

from uts_sdk.errors import (
    DecodeError,
    EncodeError,
    ErrorCode,
    RemoteError,
    UTSError,
    VerifyError,
)


class TestErrorCode:
    def test_error_codes_exist(self) -> None:
        assert ErrorCode.GENERAL_ERROR.value > 0
        assert ErrorCode.BAD_MAGIC.value > 0
        assert ErrorCode.UNKNOWN_OP.value > 0
        assert ErrorCode.INVALID_STRUCTURE.value > 0
        assert ErrorCode.REMOTE_ERROR.value > 0


class TestUTSError:
    def test_basic_error(self) -> None:
        err = UTSError(ErrorCode.GENERAL_ERROR, "test error")
        assert err.code == ErrorCode.GENERAL_ERROR
        assert "test error" in str(err)

    def test_error_with_offset(self) -> None:
        err = UTSError(ErrorCode.BAD_MAGIC, "bad magic", offset=10)
        assert err.offset == 10

    def test_error_with_context(self) -> None:
        err = UTSError(
            ErrorCode.REMOTE_ERROR,
            "remote error",
            context={"url": "https://example.com"},
        )
        assert err.context["url"] == "https://example.com"

    def test_error_repr(self) -> None:
        err = UTSError(ErrorCode.GENERAL_ERROR, "test", offset=5)
        repr_str = repr(err)
        assert "UTSError" in repr_str
        assert "GENERAL_ERROR" in repr_str
        assert "offset=5" in repr_str


class TestEncodeError:
    def test_encode_error(self) -> None:
        err = EncodeError(ErrorCode.OVERFLOW, "value too large")
        assert err.code == ErrorCode.OVERFLOW
        assert isinstance(err, UTSError)


class TestDecodeError:
    def test_decode_error(self) -> None:
        err = DecodeError(ErrorCode.UNEXPECTED_EOF, "unexpected end", offset=100)
        assert err.code == ErrorCode.UNEXPECTED_EOF
        assert err.offset == 100
        assert isinstance(err, UTSError)


class TestRemoteError:
    def test_remote_error(self) -> None:
        err = RemoteError("connection failed", context={"host": "example.com"})
        assert err.code == ErrorCode.REMOTE_ERROR
        assert "connection failed" in str(err)
        assert err.context["host"] == "example.com"

    def test_remote_error_is_uts_error(self) -> None:
        err = RemoteError("test")
        assert isinstance(err, UTSError)


class TestVerifyError:
    def test_verify_error(self) -> None:
        err = VerifyError(ErrorCode.ATTESTATION_MISMATCH, "hash mismatch")
        assert err.code == ErrorCode.ATTESTATION_MISMATCH
        assert isinstance(err, UTSError)
