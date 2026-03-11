"""Tests for types module."""

from __future__ import annotations

import pytest

from uts_sdk import (
    AttestationStatusKind,
    BitcoinAttestation,
    EASAttestation,
    EASTimestamped,
    OpCode,
    PendingAttestation,
    UnknownAttestation,
    UpgradeStatus,
    VerifyStatus,
)
from uts_sdk._types.attestations import attestation_kind
from uts_sdk._types.digest import DigestHeader, DigestOp
from uts_sdk._types.ops import DIGEST_OPS, SECURE_DIGEST_OPS
from uts_sdk._types.timestamp_steps import ForkStep


class TestPendingAttestation:
    def test_valid_url(self) -> None:
        att = PendingAttestation(url="https://calendar.example.com")
        assert att.url == "https://calendar.example.com"
        assert att.kind == "pending"

    def test_url_exceeds_max_length(self) -> None:
        with pytest.raises(ValueError, match="URL exceeds maximum"):
            PendingAttestation(url="https://example.com/" + "a" * 1000)


class TestBitcoinAttestation:
    def test_valid_height(self) -> None:
        att = BitcoinAttestation(height=800000)
        assert att.height == 800000
        assert att.kind == "bitcoin"

    def test_negative_height(self) -> None:
        with pytest.raises(ValueError, match="non-negative"):
            BitcoinAttestation(height=-1)


class TestEASAttestation:
    def test_valid_uid(self) -> None:
        uid = b"\x00" * 32
        att = EASAttestation(chain_id=1, uid=uid)
        assert att.chain_id == 1
        assert att.uid == uid
        assert att.kind == "eas-attestation"

    def test_invalid_uid_length(self) -> None:
        with pytest.raises(ValueError, match="32 bytes"):
            EASAttestation(chain_id=1, uid=b"\x00" * 16)


class TestEASTimestamped:
    def test_creation(self) -> None:
        att = EASTimestamped(chain_id=1)
        assert att.chain_id == 1
        assert att.kind == "eas-timestamped"


class TestUnknownAttestation:
    def test_creation(self) -> None:
        att = UnknownAttestation(tag=b"\x00" * 8, data=b"test")
        assert att.tag == b"\x00" * 8
        assert att.data == b"test"
        assert att.kind == "unknown"


class TestAttestationKind:
    def test_attestation_kind_pending(self) -> None:
        att = PendingAttestation(url="https://example.com")
        assert attestation_kind(att) == "pending"

    def test_attestation_kind_bitcoin(self) -> None:
        att = BitcoinAttestation(height=100)
        assert attestation_kind(att) == "bitcoin"

    def test_attestation_kind_eas_attestation(self) -> None:
        att = EASAttestation(chain_id=1, uid=b"\x00" * 32)
        assert attestation_kind(att) == "eas-attestation"

    def test_attestation_kind_eas_timestamped(self) -> None:
        att = EASTimestamped(chain_id=1)
        assert attestation_kind(att) == "eas-timestamped"

    def test_attestation_kind_unknown(self) -> None:
        att = UnknownAttestation(tag=b"\x00" * 8, data=b"test")
        assert attestation_kind(att) == "unknown"


class TestDigestHeader:
    def test_sha256_valid(self) -> None:
        digest = b"\x00" * 32
        header = DigestHeader(kind=DigestOp.SHA256, digest=digest)
        assert header.digest == digest

    def test_sha256_invalid_length(self) -> None:
        with pytest.raises(ValueError, match="Digest length mismatch"):
            DigestHeader(kind=DigestOp.SHA256, digest=b"\x00" * 16)

    def test_ripemd160_valid(self) -> None:
        digest = b"\x00" * 20
        header = DigestHeader(kind=DigestOp.RIPEMD160, digest=digest)
        assert header.digest == digest


class TestForkStep:
    def test_valid_fork(self) -> None:
        from uts_sdk import AttestationStep, PendingAttestation, SHA256Step, Timestamp

        ts1: Timestamp = [SHA256Step()]
        ts2: Timestamp = [
            AttestationStep(attestation=PendingAttestation(url="https://example.com"))
        ]

        fork = ForkStep(steps=[ts1, ts2])
        assert len(fork.steps) == 2

    def test_single_branch_fails(self) -> None:
        from uts_sdk import SHA256Step, Timestamp

        ts: Timestamp = [SHA256Step()]
        with pytest.raises(ValueError, match="at least 2 branches"):
            ForkStep(steps=[ts])


class TestOpCode:
    def test_digest_ops(self) -> None:
        assert OpCode.SHA256 in DIGEST_OPS
        assert OpCode.KECCAK256 in DIGEST_OPS
        assert OpCode.APPEND not in DIGEST_OPS

    def test_secure_digest_ops(self) -> None:
        assert OpCode.SHA256 in SECURE_DIGEST_OPS
        assert OpCode.KECCAK256 in SECURE_DIGEST_OPS
        assert OpCode.SHA1 not in SECURE_DIGEST_OPS


class TestEnums:
    def test_verify_status_values(self) -> None:
        assert VerifyStatus.VALID.value == "VALID"
        assert VerifyStatus.PENDING.value == "PENDING"

    def test_upgrade_status_values(self) -> None:
        assert UpgradeStatus.UPGRADED.value == "UPGRADED"
        assert UpgradeStatus.FAILED.value == "FAILED"

    def test_attestation_status_kind_values(self) -> None:
        assert AttestationStatusKind.VALID.value == "VALID"
        assert AttestationStatusKind.UNKNOWN.value == "UNKNOWN"
