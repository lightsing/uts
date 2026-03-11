# packages/sdk-py/src/uts_sdk/_codec/__init__.py
"""Binary codec for OpenTimestamps format."""

from __future__ import annotations

from .constants import (
    BITCOIN_TAG,
    DIGEST_LENGTHS,
    EAS_ATTEST_TAG,
    EAS_TIMESTAMPED_TAG,
    MAGIC_BYTES,
    MAX_URI_LEN,
    PENDING_TAG,
    TAG_SIZE,
)
from .decoder import Decoder
from .encoder import Encoder

__all__ = [
    "BITCOIN_TAG",
    "EAS_ATTEST_TAG",
    "EAS_TIMESTAMPED_TAG",
    "MAGIC_BYTES",
    "MAX_URI_LEN",
    "PENDING_TAG",
    "TAG_SIZE",
    "DIGEST_LENGTHS",
    "Decoder",
    "Encoder",
]
