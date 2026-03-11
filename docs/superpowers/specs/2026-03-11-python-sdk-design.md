# UTS Python SDK Design

## Overview

This document specifies the design for `uts-python-sdk`, a Python SDK for the Universal Timestamps (UTS) protocol. The SDK provides full feature parity with the existing TypeScript SDK (`@uts/sdk`), following modern Python best practices and idioms.

## Design Decisions

| Decision       | Choice                                        | Rationale                                                       |
| -------------- | --------------------------------------------- | --------------------------------------------------------------- |
| Python version | 3.10+                                         | Matches existing `pyproject.toml`, provides good typing support |
| API style      | Async-first                                   | Efficient for I/O-bound operations (stamp, verify, upgrade)     |
| HTTP client    | httpx                                         | Modern, async-native, better connection pooling                 |
| Ethereum lib   | web3.py                                       | Most popular, comprehensive EAS support                         |
| Packaging      | Poetry                                        | Already in `pyproject.toml`                                     |
| Type safety    | Full strict typing                            | `strict = true` in pyproject.toml, mypy CI enforcement          |
| Structure      | Private modules with `__init__.py` re-exports | Pythonic, hides implementation details                          |

## Package Structure

```
packages/sdk-py/
├── pyproject.toml
├── README.md
├── tests/
│   ├── __init__.py
│   ├── conftest.py
│   ├── test_codec.py
│   ├── test_merkle.py
│   └── test_sdk.py
└── src/
    └── uts_sdk/
        ├── __init__.py
        ├── _types/
        │   ├── __init__.py
        │   ├── ops.py
        │   ├── attestations.py
        │   ├── timestamp.py
        │   └── status.py
        ├── _codec/
        │   ├── __init__.py
        │   ├── constants.py
        │   ├── encoder.py
        │   └── decoder.py
        ├── _crypto/
        │   ├── __init__.py
        │   └── merkle.py
        ├── _rpc/
        │   ├── __init__.py
        │   └── bitcoin.py
        ├── _ethereum/
        │   ├── __init__.py
        │   └── eas.py
        ├── errors.py
        └── sdk.py
```

## Core Types

### Operations (enums)

```python
from enum import Enum, unique

@unique
class Op(Enum):
    SHA1 = 0x02
    RIPEMD160 = 0x03
    SHA256 = 0x08
    KECCAK256 = 0x67
    APPEND = 0xf0
    PREPEND = 0xf1
    REVERSE = 0xf2
    HEXLIFY = 0xf3
    ATTESTATION = 0x00
    FORK = 0xff
```

### Attestations (frozen dataclasses)

```python
from dataclasses import dataclass
from typing import Literal

@dataclass(frozen=True, slots=True)
class PendingAttestation:
    kind: Literal["pending"] = "pending"
    url: str

@dataclass(frozen=True, slots=True)
class BitcoinAttestation:
    kind: Literal["bitcoin"] = "bitcoin"
    height: int

@dataclass(frozen=True, slots=True)
class EASAttestation:
    kind: Literal["eas-attestation"] = "eas-attestation"
    chain: int
    uid: bytes

@dataclass(frozen=True, slots=True)
class EASTimestamped:
    kind: Literal["eas-timestamped"] = "eas-timestamped"
    chain: int

Attestation = PendingAttestation | BitcoinAttestation | EASAttestation | EASTimestamped
```

### Status Types

```python
from enum import Enum, auto

class NodePosition(Enum):
    LEFT = "left"    # sibling is right child, APPEND sibling hash
    RIGHT = "right"  # sibling is left child, PREPEND sibling hash

class VerifyStatus(Enum):
    VALID = "valid"
    PARTIAL_VALID = "partial_valid"
    INVALID = "invalid"
    PENDING = "pending"

class UpgradeStatus(Enum):
    UPGRADED = "upgraded"
    PENDING = "pending"
    FAILED = "failed"

class StampPhase(Enum):
    GENERATING_NONCE = "generating_nonce"
    BUILDING_MERKLE_TREE = "building_merkle_tree"
    BROADCASTING = "broadcasting"
    CALENDAR_RESPONSE = "calendar_response"
    BUILDING_PROOF = "building_proof"
    COMPLETE = "complete"

@dataclass(frozen=True, slots=True)
class AttestationStatus:
    attestation: Attestation
    status: VerifyStatus
    error: Exception | None = None
    additional_info: dict[str, object] | None = None

@dataclass(frozen=True, slots=True)
class UpgradeResult:
    status: UpgradeStatus
    original: PendingAttestation
    upgraded: Timestamp | None = None
    error: Exception | None = None
```

### Timestamp Steps

```python
@dataclass(frozen=True, slots=True)
class AppendStep:
    op: Literal[Op.APPEND]
    data: bytes

@dataclass(frozen=True, slots=True)
class PrependStep:
    op: Literal[Op.PREPEND]
    data: bytes

@dataclass(frozen=True, slots=True)
class HashStep:
    op: Literal[Op.SHA256, Op.KECCAK256, Op.SHA1, Op.RIPEMD160]

@dataclass(frozen=True, slots=True)
class AttestationStep:
    op: Literal[Op.ATTESTATION]
    attestation: Attestation

@dataclass(frozen=True, slots=True)
class ForkStep:
    op: Literal[Op.FORK]
    steps: tuple[Timestamp, ...]

Step = AppendStep | PrependStep | HashStep | AttestationStep | ForkStep
Timestamp = list[Step]

@dataclass(frozen=True, slots=True)
class DigestHeader:
    kind: Op
    digest: bytes

@dataclass(frozen=True, slots=True)
class DetachedTimestamp:
    header: DigestHeader
    timestamp: Timestamp
```

## Codec

### Constants

```python
DIGEST_LENGTHS: dict[Op, int] = {
    Op.SHA1: 20,
    Op.RIPEMD160: 20,
    Op.SHA256: 32,
    Op.KECCAK256: 32,
}

MAGIC_BYTES = b"\x00OpenTimestamps\x00\x00Proof\x00\xbf\x89\xe2\xe8\x84\xe8\x92\x94"

BITCOIN_ATTESTATION_TAG = b"\x05\x88\x96\x0d\x73\xd7\x19\x01"
PENDING_ATTESTATION_TAG = b"\x83\xdf\xe3\x0d\x2e\xf9\x0c\x8e"
EAS_ATTEST_TAG = b"\x8b\xf4\x6b\xf4\xcf\xd6\x74\xfa"
EAS_TIMESTAMPED_TAG = b"\x5a\xaf\xce\xeb\x1c\x7a\xd5\x8e"

MAX_URI_LEN = 1000
```

### Encoder

```python
class Encoder:
    def __init__(self, initial_size: int = 1024) -> None: ...

    def write_byte(self, byte: int) -> Self: ...
    def write_bytes(self, data: bytes) -> Self: ...
    def write_u32(self, value: int) -> Self: ...  # LEB128
    def write_op(self, op: Op) -> Self: ...
    def write_header(self, header: DigestHeader) -> Self: ...
    def write_step(self, step: Step) -> Self: ...
    def write_timestamp(self, timestamp: Timestamp) -> Self: ...

    def to_bytes(self) -> bytes: ...

    @staticmethod
    def encode_detached(ots: DetachedTimestamp) -> bytes: ...
```

### Decoder

```python
class Decoder:
    def __init__(self, data: bytes) -> None: ...

    @property
    def remaining(self) -> int: ...

    def read_byte(self) -> int: ...
    def read_bytes(self, length: int) -> bytes: ...
    def read_u32(self) -> int: ...  # LEB128
    def read_op(self) -> Op: ...
    def peek_op(self) -> Op | None: ...
    def read_header(self) -> DigestHeader: ...
    def read_step(self) -> Step: ...
    def read_timestamp(self) -> Timestamp: ...
    def read_detached(self) -> DetachedTimestamp: ...
```

## Merkle Tree

```python
from typing import Protocol
from collections.abc import Sequence

class HashFunction(Protocol):
    def __call__(self, data: bytes) -> bytes: ...

@dataclass(frozen=True, slots=True)
class SiblingNode:
    position: NodePosition
    sibling: bytes

class MerkleProof(Sequence[SiblingNode]):
    def __init__(self, siblings: list[SiblingNode]) -> None: ...
    def __len__(self) -> int: ...
    def __getitem__(self, index: int) -> SiblingNode: ...

class UnorderedMerkleTree:
    @classmethod
    def from_leaves(
        cls,
        leaves: Sequence[bytes],
        hash_func: HashFunction,
    ) -> Self: ...

    @property
    def root(self) -> bytes: ...

    @property
    def leaves(self) -> tuple[bytes, ...]: ...

    def __contains__(self, leaf: bytes) -> bool: ...

    def proof_for(self, leaf: bytes) -> MerkleProof | None: ...

    def __bytes__(self) -> bytes: ...

    @classmethod
    def from_bytes(cls, data: bytes, hash_func: HashFunction) -> Self: ...
```

## Main SDK Class

```python
class SDK:
    def __init__(
        self,
        *,
        calendars: Sequence[str] | None = None,
        btc_rpc_url: str = "https://bitcoin-rpc.publicnode.com",
        eth_rpc_urls: Mapping[int, str] | None = None,
        timeout: float = 10.0,
        quorum: int | None = None,
        nonce_size: int = 32,
        hash_algorithm: Literal["sha256", "keccak256"] = "keccak256",
    ) -> None: ...

    async def stamp(
        self,
        *digests: DigestHeader | bytes,
        on_progress: Callable[[StampPhase, float], Awaitable[None]] | None = None,
    ) -> list[DetachedTimestamp]: ...

    async def upgrade(
        self,
        stamp: DetachedTimestamp,
        *,
        keep_pending: bool = False,
    ) -> list[UpgradeResult]: ...

    async def verify(
        self,
        stamp: DetachedTimestamp,
    ) -> VerificationResult: ...

    @classmethod
    def from_env(cls) -> SDK: ...

@dataclass(frozen=True, slots=True)
class VerificationResult:
    status: VerifyStatus
    attestations: list[AttestationStatus]

    @property
    def is_valid(self) -> bool: ...
    @property
    def is_pending(self) -> bool: ...
```

## Error Handling

```python
class ErrorCode(Enum):
    GENERAL_ERROR = auto()
    BAD_MAGIC = auto()
    UNKNOWN_OP = auto()
    INVALID_STRUCTURE = auto()
    NEGATIVE_LEB128_INPUT = auto()
    OVERFLOW = auto()
    INVALID_URI = auto()
    LENGTH_MISMATCH = auto()
    UNEXPECTED_EOF = auto()
    REMOTE_ERROR = auto()
    UNSUPPORTED_ATTESTATION = auto()
    ATTESTATION_MISMATCH = auto()

class UTSError(Exception):
    def __init__(
        self,
        code: ErrorCode,
        message: str,
        *,
        offset: int | None = None,
        context: dict[str, object] | None = None,
    ) -> None: ...

class EncodeError(UTSError): ...
class DecodeError(UTSError): ...
class RemoteError(UTSError): ...
class VerifyError(UTSError): ...
```

## Dependencies

```toml
[project]
dependencies = [
    "httpx>=0.27.0",
    "web3>=7.0.0",
    "yarl>=1.9.0",  # URL handling
]

[project.optional-dependencies]
dev = [
    "pytest>=8.0.0",
    "pytest-asyncio>=0.23.0",
    "pytest-cov>=5.0.0",
    "mypy>=1.9.0",
    "ruff>=0.4.0",
]
```

## Testing Strategy

1. **Unit tests**: Each module tested in isolation
2. **Codec round-trip**: Encode → decode → compare for all types
3. **Integration tests**: Stamp/verify against test calendar
4. **Fixture tests**: Decode existing `.ots` files from TS SDK fixtures

## Migration from TypeScript SDK

| TypeScript            | Python                                  |
| --------------------- | --------------------------------------- |
| `Uint8Array`          | `bytes`                                 |
| `URL`                 | `str` (or `yarl.URL`)                   |
| `enum`                | `Enum`                                  |
| `type Union = A \| B` | `Union = A \| B` (3.10+ syntax)         |
| `interface`           | `@dataclass(frozen=True, slots=True)`   |
| `class extends`       | `class(Protocol)` for structural typing |

## Public API Surface

```python
# uts_sdk/__init__.py
from uts_sdk._types import (
    Op,
    Attestation,
    PendingAttestation,
    BitcoinAttestation,
    EASAttestation,
    EASTimestamped,
    Timestamp,
    Step,
    DetachedTimestamp,
    DigestHeader,
    VerifyStatus,
    AttestationStatus,
    UpgradeStatus,
    UpgradeResult,
    StampPhase,
    NodePosition,
)
from uts_sdk._codec import Encoder, Decoder
from uts_sdk._crypto import UnorderedMerkleTree
from uts_sdk.errors import UTSError, EncodeError, DecodeError, RemoteError, VerifyError
from uts_sdk.sdk import SDK, VerificationResult

__all__ = [
    "SDK",
    "Op",
    "Attestation",
    "PendingAttestation",
    "BitcoinAttestation",
    "EASAttestation",
    "EASTimestamped",
    "Timestamp",
    "Step",
    "DetachedTimestamp",
    "DigestHeader",
    "VerifyStatus",
    "AttestationStatus",
    "UpgradeStatus",
    "UpgradeResult",
    "StampPhase",
    "NodePosition",
    "Encoder",
    "Decoder",
    "UnorderedMerkleTree",
    "UTSError",
    "EncodeError",
    "DecodeError",
    "RemoteError",
    "VerifyError",
    "VerificationResult",
]
```
