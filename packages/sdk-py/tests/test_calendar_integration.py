"""Integration tests for calendar server operations.

These tests make real network calls to calendar servers.
Set UTS_CALENDARS environment variable or use defaults.
"""

from __future__ import annotations

import hashlib
import os

import pytest

from uts_sdk import (
    SDK,
    AttestationStatusKind,
    DetachedTimestamp,
    DigestHeader,
    DigestOp,
    PendingAttestation,
    UpgradeStatus,
    VerifyStatus,
)

DEFAULT_TEST_CALENDAR = "https://alice.btc.calendar.opentimestamps.org"


@pytest.fixture
def sdk() -> SDK:
    """Create an SDK instance with real calendar servers."""
    calendars = os.environ.get("UTS_CALENDARS", DEFAULT_TEST_CALENDAR)
    return SDK(calendars=calendars.split(","), timeout=30.0)


@pytest.fixture
def test_digest() -> bytes:
    """Create a test digest."""
    return hashlib.sha256(b"Python SDK integration test").digest()


class TestCalendarIntegration:
    """Real integration tests for calendar server operations."""

    @pytest.mark.asyncio
    async def test_stamp_and_verify(self, sdk: SDK, test_digest: bytes) -> None:
        """Test stamping a digest and verifying the result."""
        header = DigestHeader(kind=DigestOp.SHA256, digest=test_digest)

        results = await sdk.stamp(header)

        assert len(results) == 1
        stamp = results[0]
        assert isinstance(stamp, DetachedTimestamp)
        assert stamp.header.digest == test_digest

        result = await sdk.verify(stamp)

        assert result.status == VerifyStatus.PENDING
        assert len(result.attestations) >= 1
        assert all(
            a.status == AttestationStatusKind.PENDING for a in result.attestations
        )

    @pytest.mark.asyncio
    async def test_stamp_multiple_digests(self, sdk: SDK) -> None:
        """Test stamping multiple digests at once."""
        digests = [hashlib.sha256(f"test data {i}".encode()).digest() for i in range(3)]

        results = await sdk.stamp(*digests)

        assert len(results) == 3
        for i, stamp in enumerate(results):
            assert stamp.header.digest == digests[i]

    @pytest.mark.asyncio
    async def test_stamp_bytes_input(self, sdk: SDK) -> None:
        """Test stamping with raw bytes input (not DigestHeader)."""
        digest = hashlib.sha256(b"raw bytes test").digest()

        results = await sdk.stamp(digest)

        assert len(results) == 1
        assert results[0].header.digest == digest

    @pytest.mark.asyncio
    async def test_upgrade_pending_attestation(
        self, sdk: SDK, test_digest: bytes
    ) -> None:
        """Test upgrading a pending attestation (will likely stay pending)."""
        header = DigestHeader(kind=DigestOp.SHA256, digest=test_digest)

        results = await sdk.stamp(header)
        stamp = results[0]

        upgrade_results = await sdk.upgrade(stamp)

        assert len(upgrade_results) >= 1
        for result in upgrade_results:
            assert result.status in (
                UpgradeStatus.PENDING,
                UpgradeStatus.UPGRADED,
                UpgradeStatus.FAILED,
            )

    @pytest.mark.asyncio
    async def test_roundtrip_encoding(self, sdk: SDK, test_digest: bytes) -> None:
        """Test that stamps can be encoded and decoded."""
        from uts_sdk._codec import Decoder, Encoder

        header = DigestHeader(kind=DigestOp.SHA256, digest=test_digest)

        results = await sdk.stamp(header)
        stamp = results[0]

        encoded = Encoder.encode_detached(stamp)
        assert len(encoded) > 0

        decoded = Decoder.decode_detached(encoded, strict=True)
        assert decoded.header.digest == test_digest

    @pytest.mark.asyncio
    async def test_context_manager(self, test_digest: bytes) -> None:
        """Test SDK as async context manager."""
        async with SDK() as sdk:
            results = await sdk.stamp(test_digest)
            assert len(results) == 1

    @pytest.mark.asyncio
    async def test_verify_pending_attestation(self, sdk: SDK) -> None:
        """Test verifying a manually created pending attestation."""
        digest = hashlib.sha256(b"manual pending test").digest()
        header = DigestHeader(kind=DigestOp.SHA256, digest=digest)

        pending = PendingAttestation(
            url="https://alice.btc.calendar.opentimestamps.org"
        )
        from uts_sdk import AttestationStep

        step = AttestationStep(attestation=pending)
        stamp = DetachedTimestamp(header=header, timestamp=[step])

        result = await sdk.verify(stamp)

        assert result.status == VerifyStatus.PENDING
        assert result.is_pending
        assert not result.is_valid
