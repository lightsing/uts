# packages/sdk-py/src/uts_sdk/_types/status.py
"""Status enums and result types for timestamp verification."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import TYPE_CHECKING, Any

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
class AttestationStatus:
    """Result of attestation verification."""

    attestation: Attestation
    status: AttestationStatusKind
    error: Exception | None = None
    additional_info: dict[str, Any] | None = None


@dataclass(frozen=True, slots=True, kw_only=True)
class UpgradeResult:
    """Result of upgrading a pending attestation."""

    status: UpgradeStatus
    original: Attestation
    upgraded: Timestamp | None = None
    error: Exception | None = None
