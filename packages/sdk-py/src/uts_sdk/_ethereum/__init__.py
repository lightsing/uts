# packages/sdk-py/src/uts_sdk/_ethereum/__init__.py
"""Ethereum integration for UTS SDK."""

from __future__ import annotations

from .eas import (
    EAS_ABI,
    EAS_SCHEMA_ID,
    NO_EXPIRATION,
    OnChainAttestation,
    decode_content_hash,
    read_eas_attestation,
    read_eas_timestamp,
)

__all__ = [
    "EAS_ABI",
    "EAS_SCHEMA_ID",
    "NO_EXPIRATION",
    "OnChainAttestation",
    "decode_content_hash",
    "read_eas_attestation",
    "read_eas_timestamp",
]
