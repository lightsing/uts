# packages/sdk-py/src/uts_sdk/__init__.py
"""Universal Timestamps Python SDK."""

from __future__ import annotations

from uts_sdk._codec import Decoder, Encoder
from uts_sdk._crypto import MerkleProof, SiblingNode, UnorderedMerkleTree, keccak256, sha256
from uts_sdk._types import (
    Attestation,
    AttestationStatus,
    AttestationStatusKind,
    BitcoinAttestation,
    DetachedTimestamp,
    DigestHeader,
    DigestOp,
    EASTimestamped,
    EASAttestation,
    NodePosition,
    OpCode,
    PendingAttestation,
    StampPhase,
    Step,
    Timestamp,
    UnknownAttestation,
    UpgradeResult,
    UpgradeStatus,
    VerifyStatus,
    AttestationStep,
)
from uts_sdk.errors import DecodeError, EncodeError, RemoteError, UTSError, VerifyError
from uts_sdk.sdk import SDK, VerificationResult

__version__ = "0.1.0"

__all__ = [
    "SDK",
    "VerificationResult",
    "OpCode",
    "Attestation",
    "PendingAttestation",
    "BitcoinAttestation",
    "EASAttestation",
    "EASTimestamped",
    "UnknownAttestation",
    "Timestamp",
    "Step",
    "DetachedTimestamp",
    "DigestHeader",
    "DigestOp",
    "VerifyStatus",
    "AttestationStatus",
    "AttestationStatusKind",
    "UpgradeStatus",
    "UpgradeResult",
    "StampPhase",
    "NodePosition",
    "AttestationStep",
    "Encoder",
    "Decoder",
    "UnorderedMerkleTree",
    "MerkleProof",
    "SiblingNode",
    "sha256",
    "keccak256",
    "UTSError",
    "EncodeError",
    "DecodeError",
    "RemoteError",
    "VerifyError",
]
