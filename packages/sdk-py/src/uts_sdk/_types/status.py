# packages/sdk-py/src/uts_sdk/_types/status.py
"""Status enums and result types for timestamp verification."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import TYPE_CHECKING, Any, Literal

from .attestations import Attestation

if TYPE_CHECKING:
    from .timestamp_steps import Timestamp


class VerifyStatus(str, Enum):
    """Overall verification status for a timestamp."""

    VALID = "VALID"
    PARTIAL_VALID = "PARTIAL_VALID"
    INVALID = "INVALID"
    PENDING = "PENDING"


class UpgradeStatus(str, Enum):
    """Status of upgrading a pending attestation."""

    UPGRADED = "UPGRADED"
    PENDING = "PENDING"
    FAILED = "FAILED"


class AttestationStatusKind(str, Enum):
    """Status of a single attestation verification."""

    VALID = "VALID"
    INVALID = "INVALID"
    PENDING = "PENDING"
    UNKNOWN = "UNKNOWN"


class StampPhase(str, Enum):
    """Phase of a stamping operation."""

    QUEUED = "QUEUED"
    BATCHING = "BATCHING"
    AGGREGATING = "AGGREGATING"
    ATTESTING = "ATTESTING"
    CONFIRMING = "CONFIRMING"
    COMPLETED = "COMPLETED"
    FAILED = "FAILED"


class NodePosition(str, Enum):
    """Position of a node in a Merkle tree."""

    LEFT = "LEFT"
    RIGHT = "RIGHT"


@dataclass(frozen=True, slots=True, kw_only=True)
class AttestationStatusValid:
    """Result of a successful or pending attestation verification."""

    attestation: Attestation
    status: Literal[AttestationStatusKind.VALID, AttestationStatusKind.PENDING]
    additional_info: dict[str, Any] | None = None


@dataclass(frozen=True, slots=True, kw_only=True)
class AttestationStatusError:
    """Result of a failed attestation verification."""

    attestation: Attestation
    status: Literal[AttestationStatusKind.INVALID, AttestationStatusKind.UNKNOWN]
    error: Exception


AttestationStatus = AttestationStatusValid | AttestationStatusError


@dataclass(frozen=True, slots=True, kw_only=True)
class UpgradeResultUpgraded:
    """Result of a successful upgrade from pending to confirmed."""

    original: Attestation
    upgraded: Timestamp
    status: Literal[UpgradeStatus.UPGRADED] = UpgradeStatus.UPGRADED


@dataclass(frozen=True, slots=True, kw_only=True)
class UpgradeResultPending:
    """Result when attestation is still pending."""

    original: Attestation
    status: Literal[UpgradeStatus.PENDING] = UpgradeStatus.PENDING


@dataclass(frozen=True, slots=True, kw_only=True)
class UpgradeResultFailed:
    """Result when upgrade failed."""

    original: Attestation
    error: Exception
    status: Literal[UpgradeStatus.FAILED] = UpgradeStatus.FAILED


UpgradeResult = UpgradeResultUpgraded | UpgradeResultPending | UpgradeResultFailed
