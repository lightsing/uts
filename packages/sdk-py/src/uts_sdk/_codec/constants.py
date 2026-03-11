# packages/sdk-py/src/uts_sdk/_codec/constants.py
"""Binary protocol constants for OpenTimestamps format."""

from __future__ import annotations

from typing import Final

MAGIC_BYTES: Final[bytes] = (
    b"\x00OpenTimestamps\x00\x00Proof\x00\xbf\x89\xe2\xe8\x84\xe8\x92\x94"
)

TAG_SIZE: Final[int] = 8

BITCOIN_TAG: Final[bytes] = b"\x05\x88\x96\x0d\x73\xd7\x19\x01"
PENDING_TAG: Final[bytes] = b"\x83\xdf\xe3\x0d\x2e\xf9\x0c\x8e"
EAS_ATTEST_TAG: Final[bytes] = b"\x8b\xf4\x6b\xf4\xcf\xd6\x74\xfa"
EAS_TIMESTAMPED_TAG: Final[bytes] = b"\x5a\xaf\xce\xeb\x1c\x7a\xd5\x8e"

DIGEST_LENGTHS: Final[dict[str, int]] = {
    "SHA256": 32,
    "SHA1": 20,
    "RIPEMD160": 20,
    "KECCAK256": 32,
}

MAX_URI_LEN: Final[int] = 1000
