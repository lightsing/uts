# packages/sdk-py/src/uts_sdk/_types/attestations.py
"""Attestation types representing proof that data existed at a given time."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Literal, Self

if TYPE_CHECKING:
    pass


@dataclass(frozen=True, slots=True, kw_only=True)
class PendingAttestation:
    """Attestation that is not yet confirmed, pointing to a calendar server."""

    url: str
    kind: Literal["pending"] = "pending"

    def __post_init__(self) -> None:
        if len(self.url) > 1000:
            raise ValueError(f"URL exceeds maximum length of 1000: {len(self.url)}")


@dataclass(frozen=True, slots=True, kw_only=True)
class BitcoinAttestation:
    """Attestation confirmed in a Bitcoin block."""

    height: int
    kind: Literal["bitcoin"] = "bitcoin"

    def __post_init__(self) -> None:
        if self.height < 0:
            raise ValueError(f"Block height must be non-negative: {self.height}")


@dataclass(frozen=True, slots=True, kw_only=True)
class EASAttestation:
    """Attestation via Ethereum Attestation Service attestation."""

    chain_id: int
    uid: bytes
    kind: Literal["eas-attestation"] = "eas-attestation"

    def __post_init__(self) -> None:
        if len(self.uid) != 32:
            raise ValueError(f"EAS UID must be 32 bytes, got {len(self.uid)}")


@dataclass(frozen=True, slots=True, kw_only=True)
class EASTimestamped:
    """Attestation via EAS timestamp log."""

    chain_id: int
    kind: Literal["eas-timestamped"] = "eas-timestamped"


@dataclass(frozen=True, slots=True, kw_only=True)
class UnknownAttestation:
    """Attestation with an unrecognized tag."""

    tag: bytes
    data: bytes
    kind: Literal["unknown"] = "unknown"


Attestation = (
    PendingAttestation
    | BitcoinAttestation
    | EASAttestation
    | EASTimestamped
    | UnknownAttestation
)


def attestation_kind(att: Attestation) -> str:
    """Get the kind string from an attestation."""
    match att:
        case PendingAttestation():
            return "pending"
        case BitcoinAttestation():
            return "bitcoin"
        case EASAttestation():
            return "eas-attestation"
        case EASTimestamped():
            return "eas-timestamped"
        case UnknownAttestation():
            return "unknown"
