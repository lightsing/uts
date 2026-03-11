# packages/sdk-py/src/uts_sdk/_ethereum/eas.py
"""Ethereum Attestation Service (EAS) integration."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Generic, TypeVar

from eth_typing import Address, ChecksumAddress
from web3 import AsyncBaseProvider, AsyncWeb3

NO_EXPIRATION = 0

EAS_SCHEMA_ID = "0x5c5b8b295ff43c8e442be11d569e94a4cd5476f5e23df0f71bdd408df6b9649c"

_EAS_ABI: list[dict[str, Any]] = [
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


AsyncProviderT = TypeVar("AsyncProviderT", bound=AsyncBaseProvider)


class EasContract(Generic[AsyncProviderT]):
    """Helper class for interacting with EAS contract."""

    def __init__(
        self, w3: AsyncWeb3[AsyncProviderT], eas_address: Address | ChecksumAddress
    ) -> None:
        self._w3 = w3
        self._contract = w3.eth.contract(address=eas_address, abi=_EAS_ABI)

    async def get_timestamp(self, data: bytes) -> int:
        """Read timestamp from EAS contract using getTimestamp(data)."""
        padded_data = data.ljust(32, b"\x00") if len(data) < 32 else data[:32]

        result = await self._contract.functions.getTimestamp(padded_data).call()
        return int(result)

    async def get_attestation(self, uid: bytes) -> OnChainAttestation:
        """Read attestation from EAS contract using getAttestation(uid)."""
        if len(uid) != 32:
            raise ValueError(f"UID must be 32 bytes, got {len(uid)}")

        result = await self._contract.functions.getAttestation(uid).call()

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
