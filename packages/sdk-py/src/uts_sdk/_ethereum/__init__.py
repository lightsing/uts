# packages/sdk-py/src/uts_sdk/_ethereum/__init__.py
"""Ethereum integration for UTS SDK."""

from __future__ import annotations

from .eas import EAS_SCHEMA_ID, NO_EXPIRATION, EasContract, OnChainAttestation

__all__ = [
    "EasContract",
    "EAS_SCHEMA_ID",
    "NO_EXPIRATION",
    "OnChainAttestation",
]
