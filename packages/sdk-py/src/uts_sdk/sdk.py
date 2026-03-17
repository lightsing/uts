# packages/sdk-py/src/uts_sdk/sdk.py
"""Universal Timestamps SDK for Python."""

from __future__ import annotations

import secrets
from collections.abc import Awaitable, Callable, Mapping, Sequence
from dataclasses import dataclass
from types import TracebackType
from typing import Literal

import httpx
from eth_typing import Address, HexStr
from web3 import Web3
from yarl import URL

from uts_sdk._codec import Decoder
from uts_sdk._crypto import UnorderedMerkleTree
from uts_sdk._crypto.merkle import INTERNAL_PREFIX
from uts_sdk._crypto.utils import keccak256, sha256
from uts_sdk._ethereum import (
    EAS_SCHEMA_ID,
    NO_EXPIRATION,
    EasContract,
)
from uts_sdk._rpc import BitcoinRPC
from uts_sdk._types import (
    AppendStep,
    Attestation,
    AttestationStatus,
    AttestationStatusKind,
    AttestationStep,
    BitcoinAttestation,
    DetachedTimestamp,
    DigestHeader,
    DigestOp,
    EASAttestation,
    EASTimestamped,
    ForkStep,
    Keccak256Step,
    NodePosition,
    PendingAttestation,
    PrependStep,
    RIPEMD160Step,
    SHA1Step,
    SHA256Step,
    StampPhase,
    Step,
    Timestamp,
    UpgradeResult,
    UpgradeStatus,
    VerifyStatus,
)
from uts_sdk._types.timestamp_steps import HexlifyStep, ReverseStep
from uts_sdk.errors import DecodeError, ErrorCode, RemoteError, VerifyError

DEFAULT_CALENDARS = [
    "https://lgm1.calendar.test.timestamps.now/",
    # Run by Peter Todd
    "https://a.pool.opentimestamps.org/",
    "https://b.pool.opentimestamps.org/",
    # Run by Riccardo Casatta
    "https://a.pool.eternitywall.com/",
    # Run by Bull Bitcoin
    "https://ots.btc.catallaxy.com/",
]

DEFAULT_EAS_ADDRESSES: dict[int, Address] = {
    1: Address(
        Web3.to_bytes(hexstr=HexStr("0xA1207F3BBa224E2c9c3c6D5aF63D0eb1582Ce587"))
    ),
    11155111: Address(
        Web3.to_bytes(hexstr=HexStr("0xC47300428b6AD2c7D03BB76D05A176058b47E6B0"))
    ),
    534352: Address(
        Web3.to_bytes(hexstr=HexStr("0xC47300428b6AD2c7D03BB76D05A176058b47E6B0"))
    ),
    534351: Address(
        Web3.to_bytes(hexstr=HexStr("0xaEF4103A04090071165F78D45D83A0C0782c2B2a"))
    ),
}


@dataclass(frozen=True, slots=True)
class CalendarError:
    """Error from a calendar server submission."""

    url: str
    error: str


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
        hash_algorithm: Literal["sha256", "keccak256"] = "keccak256",
    ) -> None:
        self._calendars = [
            URL(str(c).rstrip("/") + "/") for c in (calendars or DEFAULT_CALENDARS)
        ]
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
            return sha256(data)
        return keccak256(data)

    async def __aenter__(self) -> SDK:
        return self

    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None:
        await self._btc_rpc.close()

    @classmethod
    def from_env(cls) -> SDK:
        """Create SDK from environment variables."""
        import os

        calendars = os.environ.get("UTS_CALENDARS")
        calendars_list = (
            [c.strip() for c in calendars.split(",")] if calendars else None
        )

        eth_rpc_urls: dict[int, str] = {}
        for key, value in os.environ.items():
            if key.startswith("UTS_ETH_RPC_URL_"):
                try:
                    chain_id = int(key[len("UTS_ETH_RPC_URL_") :])
                    eth_rpc_urls[chain_id] = value
                except ValueError:
                    pass

        timeout_str = os.environ.get("UTS_TIMEOUT", "10.0")
        quorum_str = os.environ.get("UTS_QUORUM")

        hash_algo_env = os.environ.get("UTS_HASH_ALGORITHM", "keccak256")
        if hash_algo_env == "sha256":
            hash_algorithm: Literal["sha256", "keccak256"] = "sha256"
        else:
            hash_algorithm = "keccak256"

        return cls(
            calendars=calendars_list,
            btc_rpc_url=os.environ.get(
                "UTS_BTC_RPC_URL", "https://bitcoin-rpc.publicnode.com"
            ),
            eth_rpc_urls=eth_rpc_urls or None,
            timeout=float(timeout_str),
            quorum=int(quorum_str) if quorum_str else None,
            hash_algorithm=hash_algorithm,
        )

    async def stamp(
        self,
        *digests: DigestHeader | bytes,
        on_progress: Callable[[StampPhase, float], Awaitable[None]] | None = None,
    ) -> list[DetachedTimestamp]:
        """Stamp digests by submitting to calendar servers."""
        import asyncio

        digest_headers = [
            (
                d
                if isinstance(d, DigestHeader)
                else DigestHeader(kind=DigestOp.SHA256, digest=d)
            )
            for d in digests
        ]

        if on_progress:
            await on_progress(StampPhase.QUEUED, 0.0)

        nonces = [secrets.token_bytes(self._nonce_size) for _ in digest_headers]
        nonce_digests = [
            self._hash(h.digest + n)
            for h, n in zip(digest_headers, nonces, strict=True)
        ]

        if on_progress:
            await on_progress(StampPhase.BATCHING, 0.5)

        tree = UnorderedMerkleTree.from_leaves(nonce_digests, self._hash)
        root = tree.root

        if on_progress:
            await on_progress(StampPhase.AGGREGATING, 0.5)

        calendar_errors: list[CalendarError] = []

        async def submit_to_calendar(
            calendar: URL,
        ) -> tuple[Timestamp | None, CalendarError | None]:
            """Submit root to a calendar server.

            Returns (timestamp, None) on success, (None, error) on failure.
            """
            try:
                async with httpx.AsyncClient(timeout=self._timeout) as client:
                    response = await client.post(
                        str(calendar / "digest"),
                        content=root,
                        headers={"Accept": "application/vnd.opentimestamps.v1"},
                    )
                    if response.is_success:
                        decoder = Decoder(response.content)
                        ts = decoder.read_timestamp()
                        return ts, None
                    return None, CalendarError(
                        url=str(calendar),
                        error=f"HTTP {response.status_code}: {response.text[:200]}",
                    )
            except httpx.TimeoutException as e:
                return None, CalendarError(url=str(calendar), error=f"Timeout: {e}")
            except httpx.RequestError as e:
                return None, CalendarError(
                    url=str(calendar), error=f"Network error: {e}"
                )
            except DecodeError as e:
                return None, CalendarError(
                    url=str(calendar), error=f"Decode error: {e}"
                )

        results = await asyncio.gather(
            *[submit_to_calendar(c) for c in self._calendars]
        )

        successful: list[Timestamp] = []
        for ts, err in results:
            if ts is not None:
                successful.append(ts)
            elif err is not None:
                calendar_errors.append(err)

        if len(successful) < self._quorum:
            error_details = "; ".join(f"{e.url}: {e.error}" for e in calendar_errors)
            raise RemoteError(
                f"Quorum not reached: {len(successful)}/{self._quorum} calendars responded. "
                f"Errors: {error_details}"
            )

        merged: Timestamp = (
            successful[0] if len(successful) == 1 else [ForkStep(steps=successful)]
        )

        if on_progress:
            await on_progress(StampPhase.ATTESTING, 1.0)

        # Build timestamps with proper Merkle proof steps
        result_timestamps: list[DetachedTimestamp] = []
        hash_step = (
            SHA256Step() if self._hash_algorithm == "sha256" else Keccak256Step()
        )

        for i, header in enumerate(digest_headers):
            # Start with: APPEND nonce, then HASH
            steps: list[Step] = [
                AppendStep(data=nonces[i]),
                hash_step,
            ]

            # Add Merkle proof steps with inner-node prefix
            proof = tree.proof_for(nonce_digests[i])
            if proof:
                for node in proof:
                    if node.position == NodePosition.LEFT:
                        # Sibling is right child: PREPEND prefix, APPEND sibling, HASH
                        steps.extend(
                            [
                                PrependStep(data=INTERNAL_PREFIX),
                                AppendStep(data=node.sibling),
                                hash_step,
                            ]
                        )
                    else:
                        # Sibling is left child: PREPEND sibling, PREPEND prefix, HASH
                        steps.extend(
                            [
                                PrependStep(data=node.sibling),
                                PrependStep(data=INTERNAL_PREFIX),
                                hash_step,
                            ]
                        )

            steps.extend(merged)
            result_timestamps.append(DetachedTimestamp(header=header, timestamp=steps))

        if on_progress:
            await on_progress(StampPhase.COMPLETED, 1.0)

        return result_timestamps

    async def verify(self, stamp: DetachedTimestamp) -> VerificationResult:
        """Verify a detached timestamp."""
        statuses = await self._verify_timestamp(stamp.header.digest, stamp.timestamp)
        return self._aggregate_result(statuses)

    async def upgrade(
        self,
        stamp: DetachedTimestamp,
        *,
        keep_pending: bool = False,
    ) -> list[UpgradeResult]:
        """Upgrade pending attestations in a timestamp."""
        return await self._upgrade_timestamp(
            stamp.header.digest, stamp.timestamp, keep_pending
        )

    async def _upgrade_timestamp(
        self,
        current: bytes,
        timestamp: Timestamp,
        keep_pending: bool,
    ) -> list[UpgradeResult]:
        """Recursively upgrade pending attestations."""
        results: list[UpgradeResult] = []

        for i, step in enumerate(timestamp):
            match step:
                case ForkStep(steps=branches):
                    for branch in branches:
                        branch_results = await self._upgrade_timestamp(
                            current, branch, keep_pending
                        )
                        results.extend(branch_results)
                case AttestationStep(attestation=att):
                    if isinstance(att, PendingAttestation):
                        result = await self._upgrade_pending(current, att)
                        results.append(result)
                        if result.status == UpgradeStatus.UPGRADED and result.upgraded:
                            if keep_pending:
                                timestamp[i] = ForkStep(steps=[[step], result.upgraded])
                            else:
                                timestamp[i : i + 1] = result.upgraded
                case _:
                    current = self._execute_step(current, step)

        return results

    async def _upgrade_pending(
        self, commitment: bytes, pending: PendingAttestation
    ) -> UpgradeResult:
        """Upgrade a single pending attestation."""

        commitment_hex = commitment.hex()
        try:
            async with httpx.AsyncClient(timeout=self._timeout) as client:
                response = await client.get(
                    f"{pending.url.rstrip('/')}/timestamp/{commitment_hex}",
                    headers={"Accept": "application/vnd.opentimestamps.v1"},
                )
                if response.status_code == 202 or response.status_code == 404:
                    return UpgradeResult(status=UpgradeStatus.PENDING, original=pending)
                if response.is_success:
                    decoder = Decoder(response.content)
                    ts = decoder.read_timestamp()
                    return UpgradeResult(
                        status=UpgradeStatus.UPGRADED, original=pending, upgraded=ts
                    )
                return UpgradeResult(
                    status=UpgradeStatus.FAILED,
                    original=pending,
                    error=RemoteError(f"Calendar returned {response.status_code}"),
                )
        except Exception as e:
            return UpgradeResult(status=UpgradeStatus.FAILED, original=pending, error=e)

    async def _verify_timestamp(
        self, current: bytes, timestamp: Timestamp
    ) -> list[AttestationStatus]:
        """Verify timestamp steps and return attestation statuses."""
        attestations: list[AttestationStatus] = []

        for step in timestamp:
            match step:
                case ForkStep(steps=branches):
                    for branch in branches:
                        branch_results = await self._verify_timestamp(current, branch)
                        attestations.extend(branch_results)
                case AttestationStep(attestation=att):
                    status = await self._verify_attestation(current, att)
                    attestations.append(status)
                case _:
                    current = self._execute_step(current, step)

        return attestations

    def _execute_step(self, current: bytes, step: Step) -> bytes:
        """Execute a single timestamp step."""
        import hashlib

        match step:
            case AppendStep(data=data):
                return current + data
            case PrependStep(data=data):
                return data + current
            case ReverseStep():
                return current[::-1]
            case HexlifyStep():
                return current.hex().encode()
            case SHA256Step():
                return sha256(current)
            case Keccak256Step():
                return keccak256(current)
            case SHA1Step():
                return hashlib.sha1(current).digest()
            case RIPEMD160Step():
                return hashlib.new("ripemd160", current).digest()
            case _:
                raise VerifyError(
                    ErrorCode.INVALID_STRUCTURE,
                    f"Unsupported step type: {type(step).__name__}",
                )

    async def _verify_attestation(
        self, digest: bytes, attestation: Attestation
    ) -> AttestationStatus:
        """Verify a single attestation."""
        if isinstance(attestation, PendingAttestation):
            return AttestationStatus(
                attestation=attestation,
                status=AttestationStatusKind.PENDING,
            )
        elif isinstance(attestation, BitcoinAttestation):
            return await self._verify_bitcoin(digest, attestation)
        elif isinstance(attestation, (EASAttestation, EASTimestamped)):
            return await self._verify_eas(digest, attestation)
        else:
            return AttestationStatus(
                attestation=attestation,
                status=AttestationStatusKind.UNKNOWN,
                error=VerifyError(
                    ErrorCode.UNSUPPORTED_ATTESTATION, "Unknown attestation type"
                ),
            )

    async def _verify_bitcoin(
        self, digest: bytes, att: BitcoinAttestation
    ) -> AttestationStatus:
        """Verify a Bitcoin attestation."""
        try:
            block_hash = await self._btc_rpc.get_block_hash(att.height)
            header = await self._btc_rpc.get_block_header(block_hash)

            # Bitcoin displays hashes in reversed byte order (little-endian for display)
            # The merkleroot from RPC is in display format, so reverse it for comparison
            merkleroot_bytes = bytes.fromhex(header.merkleroot)[::-1]

            if digest != merkleroot_bytes:
                return AttestationStatus(
                    attestation=att,
                    status=AttestationStatusKind.INVALID,
                    error=VerifyError(
                        ErrorCode.ATTESTATION_MISMATCH,
                        f"Merkle root mismatch at height {att.height}",
                    ),
                )

            return AttestationStatus(
                attestation=att,
                status=AttestationStatusKind.VALID,
                additional_info={"header": header},
            )
        except Exception as e:
            return AttestationStatus(
                attestation=att,
                status=AttestationStatusKind.UNKNOWN,
                error=VerifyError(ErrorCode.REMOTE_ERROR, str(e)),
            )

    async def _verify_eas(
        self, digest: bytes, att: EASAttestation | EASTimestamped
    ) -> AttestationStatus:
        """Verify an EAS attestation."""
        from web3 import AsyncWeb3

        rpc_url = self._eth_rpc_urls.get(att.chain_id)
        if not rpc_url:
            return AttestationStatus(
                attestation=att,
                status=AttestationStatusKind.UNKNOWN,
                error=VerifyError(
                    ErrorCode.GENERAL_ERROR, f"No RPC URL for chain {att.chain_id}"
                ),
            )

        eas_address = DEFAULT_EAS_ADDRESSES.get(att.chain_id)
        if not eas_address:
            return AttestationStatus(
                attestation=att,
                status=AttestationStatusKind.UNKNOWN,
                error=VerifyError(
                    ErrorCode.GENERAL_ERROR, f"No EAS address for chain {att.chain_id}"
                ),
            )

        try:
            w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider(rpc_url))
            eas = EasContract(w3, eas_address)

            if isinstance(att, EASTimestamped):
                ts = await eas.get_timestamp(digest)
                if ts == 0:
                    return AttestationStatus(
                        attestation=att,
                        status=AttestationStatusKind.INVALID,
                        error=VerifyError(
                            ErrorCode.ATTESTATION_MISMATCH, "No EAS timestamp found"
                        ),
                    )
                return AttestationStatus(
                    attestation=att,
                    status=AttestationStatusKind.VALID,
                    additional_info={"time": ts},
                )
            else:
                on_chain = await eas.get_attestation(att.uid)

                if on_chain.schema != EAS_SCHEMA_ID[2:]:
                    return AttestationStatus(
                        attestation=att,
                        status=AttestationStatusKind.INVALID,
                        error=VerifyError(
                            ErrorCode.ATTESTATION_MISMATCH, "Schema mismatch"
                        ),
                    )

                if on_chain.expiration_time != NO_EXPIRATION:
                    return AttestationStatus(
                        attestation=att,
                        status=AttestationStatusKind.INVALID,
                        error=VerifyError(
                            ErrorCode.ATTESTATION_MISMATCH, "Has expiration"
                        ),
                    )

                if on_chain.revocable:
                    return AttestationStatus(
                        attestation=att,
                        status=AttestationStatusKind.INVALID,
                        error=VerifyError(
                            ErrorCode.ATTESTATION_MISMATCH, "Is revocable"
                        ),
                    )

                return AttestationStatus(
                    attestation=att,
                    status=AttestationStatusKind.VALID,
                    additional_info={"attestation": on_chain},
                )
        except Exception as e:
            return AttestationStatus(
                attestation=att,
                status=AttestationStatusKind.UNKNOWN,
                error=VerifyError(ErrorCode.REMOTE_ERROR, str(e)),
            )

    def _aggregate_result(
        self, statuses: list[AttestationStatus]
    ) -> VerificationResult:
        """Aggregate attestation statuses into overall verification result."""
        counts = dict.fromkeys(AttestationStatusKind, 0)
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
