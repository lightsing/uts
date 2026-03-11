# packages/sdk-py/src/uts_sdk/_rpc/bitcoin.py
"""Bitcoin RPC client for verifying Bitcoin attestations."""

from __future__ import annotations

import json
from dataclasses import dataclass
from types import TracebackType
from typing import Any
from urllib.parse import urlparse

import httpx

from uts_sdk.errors import RemoteError


@dataclass(frozen=True, slots=True)
class BitcoinBlockHeader:
    """Bitcoin block header data."""

    hash: str
    confirmations: int
    height: int
    version: int
    versionHex: str
    merkleroot: str
    time: int
    mediantime: int
    nonce: int
    bits: str
    difficulty: float
    chainwork: str
    nTx: int
    previousblockhash: str | None = None
    nextblockhash: str | None = None


class BitcoinRPC:
    """Async client for Bitcoin JSON-RPC."""

    def __init__(self, url: str = "https://bitcoin-rpc.publicnode.com") -> None:
        parsed = urlparse(url)
        if not parsed.scheme or not parsed.netloc:
            raise ValueError(f"Invalid RPC URL: {url}")
        self._url = url
        self._client = httpx.AsyncClient(timeout=30.0)

    async def call(self, method: str, params: list[Any] | None = None) -> Any:
        if params is None:
            params = []

        request_body = {
            "jsonrpc": "1.0",
            "method": method,
            "params": params,
            "id": 1,
        }

        try:
            response = await self._client.post(
                self._url,
                json=request_body,
                headers={"Content-Type": "application/json"},
            )
        except httpx.RequestError as e:
            raise RemoteError(
                f"Bitcoin RPC network error: {e}",
                context={"method": method, "params": params},
            ) from e

        try:
            response_data = response.json()
        except json.JSONDecodeError as e:
            raise RemoteError(
                "Bitcoin RPC invalid JSON response",
                context={"status_code": response.status_code, "error": str(e)},
            ) from e

        if response.status_code >= 400:
            raise RemoteError(
                f"Bitcoin RPC HTTP error: {response.status_code}",
                context={
                    "status_code": response.status_code,
                    "response": response_data,
                },
            )

        if response_data.get("error") is not None:
            error = response_data["error"]
            raise RemoteError(
                f"Bitcoin RPC error: {error.get('message', 'Unknown error')}",
                context={"error_code": error.get("code"), "error_details": error},
            )

        return response_data.get("result")

    async def get_block_hash(self, height: int) -> str:
        result = await self.call("getblockhash", [height])
        return str(result)

    async def get_block_header(self, block_hash: str) -> BitcoinBlockHeader:
        result = await self.call("getblockheader", [block_hash])

        return BitcoinBlockHeader(
            hash=result["hash"],
            confirmations=result["confirmations"],
            height=result["height"],
            version=result["version"],
            versionHex=result["versionHex"],
            merkleroot=result["merkleroot"],
            time=result["time"],
            mediantime=result["mediantime"],
            nonce=result["nonce"],
            bits=result["bits"],
            difficulty=result.get("difficulty", 1.0),
            chainwork=result["chainwork"],
            nTx=result["nTx"],
            previousblockhash=result.get("previousblockhash"),
            nextblockhash=result.get("nextblockhash"),
        )

    async def close(self) -> None:
        await self._client.aclose()

    async def __aenter__(self) -> BitcoinRPC:
        return self

    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None:
        await self.close()
