# packages/sdk-py/src/uts_sdk/_ethereum/eas.py
"""Ethereum Attestation Service (EAS) integration."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

NO_EXPIRATION = 0

EAS_SCHEMA_ID = "0x5c5b8b295ff43c8e442be11d569e94a4cd5476f5e23df0f71bdd408df6b9649c"

EAS_ABI = [
    {
        "anonymous": False,
        "inputs": [
            {"indexed": True, "name": "recipient", "type": "address"},
            {"indexed": True, "name": "attester", "type": "address"},
            {"indexed": False, "name": "uid", "type": "bytes32"},
            {"indexed": False, "name": "schema", "type": "bytes32"},
        ],
        "name": "Attested",
        "type": "event",
    },
    {
        "inputs": [{"name": "uid", "type": "bytes32"}],
        "name": "getAttestation",
        "outputs": [
            {
                "components": [
                    {"name": "uid", "type": "bytes32"},
                    {"name": "schema", "type": "bytes32"},
                    {"name": "time", "type": "uint64"},
                    {"name": "expirationTime", "type": "uint64"},
                    {"name": "revocationTime", "type": "uint64"},
                    {"name": "refUID", "type": "bytes32"},
                    {"name": "recipient", "type": "address"},
                    {"name": "attester", "type": "address"},
                    {"name": "revocable", "type": "bool"},
                    {"name": "data", "type": "bytes"},
                ],
                "name": "",
                "type": "tuple",
            }
        ],
        "stateMutability": "view",
        "type": "function",
    },
    {
        "inputs": [{"name": "data", "type": "bytes32"}],
        "name": "getTimestamp",
        "outputs": [{"name": "", "type": "uint64"}],
        "stateMutability": "view",
        "type": "function",
    },
]


@dataclass(frozen=True, slots=True)
class OnChainAttestation:
    """Attestation data from EAS contract."""

    uid: str
    schema: str
    time: int
    expiration_time: int
    revocation_time: int
    ref_uid: str
    recipient: str
    attester: str
    revocable: bool
    data: str


def read_eas_timestamp(
    w3: Any,
    eas_address: str,
    data: bytes,
) -> int:
    """Read timestamp from EAS contract using getTimestamp(data)."""
    contract = w3.eth.contract(address=eas_address, abi=EAS_ABI)

    padded_data = data.ljust(32, b"\x00") if len(data) < 32 else data[:32]

    result = contract.functions.getTimestamp(padded_data).call()
    return int(result)


def read_eas_attestation(
    w3: Any,
    eas_address: str,
    uid: bytes,
) -> OnChainAttestation:
    """Read attestation from EAS contract using getAttestation(uid)."""
    contract = w3.eth.contract(address=eas_address, abi=EAS_ABI)

    if len(uid) != 32:
        raise ValueError(f"UID must be 32 bytes, got {len(uid)}")

    result = contract.functions.getAttestation(uid).call()

    return OnChainAttestation(
        uid=result[0].hex(),
        schema=result[1].hex(),
        time=int(result[2]),
        expiration_time=int(result[3]),
        revocation_time=int(result[4]),
        ref_uid=result[5].hex(),
        recipient=result[6],
        attester=result[7],
        revocable=bool(result[8]),
        data=result[9].hex() if isinstance(result[9], bytes) else result[9],
    )


def decode_content_hash(data: str | bytes) -> bytes:
    """Decode bytes32 contentHash from attestation data."""
    if isinstance(data, str):
        if data.startswith("0x"):
            data = data[2:]
        data = bytes.fromhex(data)

    if len(data) < 32:
        raise ValueError(f"Data too short for content hash: {len(data)}")

    return data[:32]
