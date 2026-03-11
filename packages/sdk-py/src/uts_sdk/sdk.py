# packages/sdk-py/src/uts_sdk/sdk.py
"""Universal Timestamps SDK for Python."""

from __future__ import annotations

import hashlib
import secrets
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any, Awaitable, Callable, Mapping, Sequence

from yarl import URL

from uts_sdk._codec import Decoder, Encoder
from uts_sdk._crypto import UnorderedMerkleTree
from uts_sdk._ethereum import (
    EAS_SCHEMA_ID,
    NO_EXPIRATION,
    decode_content_hash,
    read_eas_attestation,
    read_eas_timestamp,
)
from uts_sdk._rpc import BitcoinRPC
from uts_sdk._types import (
    AppendStep,
    Attestation,
    AttestationStep,
    AttestationStatus,
    AttestationStatusValid,
    AttestationStatusError,
    BitcoinAttestation,
    DetachedTimestamp,
    DigestHeader,
    DigestOp,
    EASTimestamped,
    EASAttestation,
    ForkStep,
    OpCode,
    PendingAttestation,
    PrependStep,
    StampPhase,
    Timestamp,
    UpgradeResult,
    UpgradeResultFailed,
    UpgradeResultPending,
    UpgradeResultUpgraded,
    UpgradeStatus,
    VerifyStatus,
)
from uts_sdk.errors import ErrorCode, RemoteError, VerifyError

if TYPE_CHECKING:
    pass

DEFAULT_CALENDARS = [
    "https://lgm1.test.timestamps.now/",
]

DEFAULT_EAS_ADDRESSES: dict[int, str] = {
    1: "0xA1207F3BBa224E2c9c3c6D5aF63D0eb1582Ce587",
    11155111: "0xC2679fBD37d54388Ce493F1DB75320D236e1815e",
    534352: "0xC47300428b6AD2c7D03BB76D05A176058b47E6B0",
    534351: "0xaEF4103A04090071165F78D45D83A0C0782c2B2a",
}


@dataclass(frozen=True, slots=True)
class VerificationResult:
    """Result of verifying a detached timestamp."""

    status: VerifyStatus
    attestations: list[AttestationStatus]

    @property
    def is_valid(self) -> bool:
        return self.status in (VerifyStatus.VALID, VerifyStatus.PARTIAL_VALID)

    @property
    def is_pending(self) -> bool:
        return self.status == VerifyStatus.PENDING


class SDK:
    """Universal Timestamps SDK for Python.

    Usage:
        async with SDK() as sdk:
            result = await sdk.stamp(digests)
            status = await sdk.verify(result[0])
    """

    def __init__(
        self,
        *,
        calendars: Sequence[str] | None = None,
        btc_rpc_url: str = "https://bitcoin-rpc.publicnode.com",
        eth_rpc_urls: Mapping[int, str] | None = None,
        timeout: float = 10.0,
        quorum: int | None = None,
        nonce_size: int = 32,
        hash_algorithm: str = "keccak256",
    ) -> None:
        self._calendars = [URL(str(c).rstrip("/") + "/") for c in (calendars or DEFAULT_CALENDARS)]
        self._btc_rpc = BitcoinRPC(btc_rpc_url)
        self._eth_rpc_urls = dict(eth_rpc_urls) if eth_rpc_urls else {}
        self._timeout = timeout
        self._nonce_size = nonce_size
        self._quorum = quorum or max(1, int(len(self._calendars) * 0.66))
        self._hash_algorithm = hash_algorithm

        if hash_algorithm not in ("sha256", "keccak256"):
            raise ValueError(f"Unsupported hash algorithm: {hash_algorithm}")

    @property
    def calendars(self) -> list[str]:
        return [str(c) for c in self._calendars]

    @property
    def timeout(self) -> float:
        return self._timeout

    @property
    def nonce_size(self) -> int:
        return self._nonce_size

    def _hash(self, data: bytes) -> bytes:
        if self._hash_algorithm == "sha256":
            return hashlib.sha256(data).digest()
        return hashlib.sha3_256(data).digest()

    async def __aenter__(self) -> SDK:
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        await self._btc_rpc.close()

    async def stamp(
        self,
        *digests: DigestHeader | bytes,
        on_progress: Callable[[StampPhase, float], Awaitable[None]] | None = None,
    ) -> list[DetachedTimestamp]:
        import asyncio
        import httpx

        digest_headers = [
            d if isinstance(d, DigestHeader) else DigestHeader(kind=DigestOp.SHA256, digest=d)
            for d in digests
        ]

        if on_progress:
            await on_progress(StampPhase.QUEUED, 0.0)

        nonces = [secrets.token_bytes(self._nonce_size) for _ in digest_headers]
        nonce_digests = [self._hash(h.digest + n) for h, n in zip(digest_headers, nonces)]

        if on_progress:
            await on_progress(StampPhase.BATCHING, 0.5)

        tree = UnorderedMerkleTree.from_leaves(nonce_digests, self._hash)
        root = tree.root

        if on_progress:
            await on_progress(StampPhase.AGGREGATING, 0.5)

        async def submit_to_calendar(calendar: URL) -> Timestamp | None:
            try:
                async with httpx.AsyncClient(timeout=self._timeout) as client:
                    response = await client.post(
                        str(calendar / "digest"),
                        content=root,
                        headers={"Accept": "application/vnd.opentimestamps.v1"},
                    )
                    if response.is_success:
                        decoder = Decoder(response.content)
                        return decoder.read_timestamp()
            except Exception:
                pass
            return None

        results = await asyncio.gather(*[submit_to_calendar(c) for c in self._calendars])
        successful = [r for r in results if r is not None]

        if len(successful) < self._quorum:
            raise RemoteError(
                f"Only {len(successful)} calendar responses, need {self._quorum}",
            )

        merged: Timestamp
        if len(successful) == 1:
            merged = successful[0]
        else:
            merged = [ForkStep(steps=successful)]

        if on_progress:
            await on_progress(StampPhase.ATTESTING, 1.0)

        result_timestamps: list[DetachedTimestamp] = []
        for i, header in enumerate(digest_headers):
            steps: list[Any] = [
                AppendStep(data=nonces[i]),
            ]

            proof = tree.proof_for(nonce_digests[i])
            if proof:
                for node in proof:
                    from uts_sdk._types.status import NodePosition

                    if node.position == NodePosition.LEFT:
                        steps.append(PrependStep(data=node.sibling))
                    else:
                        steps.append(AppendStep(data=node.sibling))

            steps.extend(merged)

            result_timestamps.append(DetachedTimestamp(header=header, timestamp=steps))

        if on_progress:
            await on_progress(StampPhase.COMPLETED, 1.0)

        return result_timestamps

    async def verify(self, stamp: DetachedTimestamp) -> VerificationResult:
        statuses = await self._verify_timestamp(stamp)
        return self._aggregate_result(statuses)

    async def _verify_timestamp(self, stamp: DetachedTimestamp) -> list[AttestationStatus]:
        current = stamp.header.digest
        statuses: list[AttestationStatus] = []

        for step in stamp.timestamp:
            if isinstance(step, AppendStep):
                current = current + step.data
            elif isinstance(step, PrependStep):
                current = step.data + current
            elif isinstance(step, AttestationStep):
                status = await self._verify_attestation(current, step.attestation)
                statuses.append(status)

        return statuses

    async def _verify_attestation(
        self, digest: bytes, attestation: Attestation
    ) -> AttestationStatus:
        if isinstance(attestation, PendingAttestation):
            return AttestationStatusValid(
                attestation=attestation,
                status="PENDING",
            )
        elif isinstance(attestation, BitcoinAttestation):
            return await self._verify_bitcoin(digest, attestation)
        elif isinstance(attestation, (EASAttestation, EASTimestamped)):
            return await self._verify_eas(digest, attestation)
        else:
            return AttestationStatusError(
                attestation=attestation,
                status="UNKNOWN",
                error=VerifyError(ErrorCode.UNSUPPORTED_ATTESTATION, "Unknown attestation type"),
            )

    async def _verify_bitcoin(self, digest: bytes, att: BitcoinAttestation) -> AttestationStatus:
        try:
            block_hash = await self._btc_rpc.get_block_hash(att.height)
            header = await self._btc_rpc.get_block_header(block_hash)

            digest_reversed = digest[::-1]
            merkleroot_bytes = bytes.fromhex(header.merkleroot)

            if digest_reversed != merkleroot_bytes:
                return AttestationStatusError(
                    attestation=att,
                    status="INVALID",
                    error=VerifyError(
                        ErrorCode.ATTESTATION_MISMATCH,
                        f"Merkle root mismatch at height {att.height}",
                    ),
                )

            return AttestationStatusValid(
                attestation=att,
                status="VALID",
                additional_info={"header": header},
            )
        except Exception as e:
            return AttestationStatusError(
                attestation=att,
                status="UNKNOWN",
                error=VerifyError(ErrorCode.REMOTE_ERROR, str(e)),
            )

    async def _verify_eas(
        self, digest: bytes, att: EASAttestation | EASTimestamped
    ) -> AttestationStatus:
        from web3 import Web3

        rpc_url = self._eth_rpc_urls.get(att.chain_id)
        if not rpc_url:
            return AttestationStatusError(
                attestation=att,
                status="UNKNOWN",
                error=VerifyError(
                    ErrorCode.GENERAL_ERROR,
                    f"No RPC URL for chain {att.chain_id}",
                ),
            )

        eas_address = DEFAULT_EAS_ADDRESSES.get(att.chain_id)
        if not eas_address:
            return AttestationStatusError(
                attestation=att,
                status="UNKNOWN",
                error=VerifyError(
                    ErrorCode.GENERAL_ERROR,
                    f"No EAS address for chain {att.chain_id}",
                ),
            )

        try:
            w3 = Web3(Web3.HTTPProvider(rpc_url))

            if isinstance(att, EASTimestamped):
                ts = read_eas_timestamp(w3, eas_address, digest)
                if ts == 0:
                    return AttestationStatusError(
                        attestation=att,
                        status="INVALID",
                        error=VerifyError(
                            ErrorCode.ATTESTATION_MISMATCH,
                            "No EAS timestamp found",
                        ),
                    )
                return AttestationStatusValid(
                    attestation=att,
                    status="VALID",
                    additional_info={"time": ts},
                )
            else:
                on_chain = read_eas_attestation(w3, eas_address, att.uid)

                if on_chain.schema != EAS_SCHEMA_ID[2:]:
                    return AttestationStatusError(
                        attestation=att,
                        status="INVALID",
                        error=VerifyError(ErrorCode.ATTESTATION_MISMATCH, "Schema mismatch"),
                    )

                if on_chain.expiration_time != NO_EXPIRATION:
                    return AttestationStatusError(
                        attestation=att,
                        status="INVALID",
                        error=VerifyError(ErrorCode.ATTESTATION_MISMATCH, "Has expiration"),
                    )

                if on_chain.revocable:
                    return AttestationStatusError(
                        attestation=att,
                        status="INVALID",
                        error=VerifyError(ErrorCode.ATTESTATION_MISMATCH, "Is revocable"),
                    )

                return AttestationStatusValid(
                    attestation=att,
                    status="VALID",
                    additional_info={"attestation": on_chain},
                )
        except Exception as e:
            return AttestationStatusError(
                attestation=att,
                status="UNKNOWN",
                error=VerifyError(ErrorCode.REMOTE_ERROR, str(e)),
            )

    def _aggregate_result(self, statuses: list[AttestationStatus]) -> VerificationResult:
        from uts_sdk._types.status import AttestationStatusKind

        counts = {k: 0 for k in AttestationStatusKind}
        for s in statuses:
            counts[s.status] += 1

        if counts[AttestationStatusKind.VALID] > 0:
            if (
                counts[AttestationStatusKind.INVALID] > 0
                or counts[AttestationStatusKind.UNKNOWN] > 0
            ):
                status = VerifyStatus.PARTIAL_VALID
            else:
                status = VerifyStatus.VALID
        elif counts[AttestationStatusKind.PENDING] > 0:
            status = VerifyStatus.PENDING
        elif counts[AttestationStatusKind.INVALID] > 0:
            status = VerifyStatus.INVALID
        else:
            status = VerifyStatus.INVALID

        return VerificationResult(status=status, attestations=statuses)
