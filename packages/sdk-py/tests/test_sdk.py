"""Tests for main SDK class."""

from __future__ import annotations

import hashlib

import pytest

from uts_sdk import (
    SDK,
    AttestationStatusKind,
    AttestationStep,
    DetachedTimestamp,
    DigestHeader,
    DigestOp,
    PendingAttestation,
    VerifyStatus,
)


def test_sdk_initialization() -> None:
    sdk = SDK()
    assert sdk is not None
    assert len(sdk.calendars) > 0


def test_sdk_options() -> None:
    sdk = SDK(
        calendars=["https://test.calendar.com"],
        timeout=20.0,
        nonce_size=64,
    )
    assert sdk.timeout == 20.0
    assert sdk.nonce_size == 64


def test_sdk_default_calendars() -> None:
    sdk = SDK()
    assert "https://lgm1.test.timestamps.now/" in sdk.calendars


def test_sdk_invalid_hash_algorithm() -> None:
    with pytest.raises(ValueError, match="Unsupported hash algorithm"):
        SDK(hash_algorithm="md5")


@pytest.mark.asyncio
async def test_sdk_context_manager() -> None:
    async with SDK() as sdk:
        assert sdk is not None


@pytest.mark.asyncio
async def test_sdk_verify_pending() -> None:
    digest = hashlib.sha256(b"test data").digest()
    header = DigestHeader(kind=DigestOp.SHA256, digest=digest)

    pending = PendingAttestation(url="https://calendar.example.com")
    step = AttestationStep(attestation=pending)
    stamp = DetachedTimestamp(header=header, timestamp=[step])

    async with SDK() as sdk:
        result = await sdk.verify(stamp)

        assert result.status == VerifyStatus.PENDING
        assert len(result.attestations) == 1
        assert result.attestations[0].status == AttestationStatusKind.PENDING
        assert not result.is_valid
        assert result.is_pending


@pytest.mark.asyncio
async def test_sdk_upgrade_pending() -> None:
    digest = hashlib.sha256(b"test data").digest()
    header = DigestHeader(kind=DigestOp.SHA256, digest=digest)

    pending = PendingAttestation(url="https://calendar.example.com")
    step = AttestationStep(attestation=pending)
    stamp = DetachedTimestamp(header=header, timestamp=[step])

    async with SDK() as sdk:
        results = await sdk.upgrade(stamp)

        assert len(results) == 1
        assert results[0].original == pending
