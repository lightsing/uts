"""Tests for Bitcoin RPC client."""

from __future__ import annotations

import pytest
from unittest.mock import AsyncMock, MagicMock, patch

from uts_sdk._rpc.bitcoin import BitcoinBlockHeader, BitcoinRPC


@pytest.mark.asyncio
async def test_bitcoin_rpc_client() -> None:
    rpc = BitcoinRPC(url="https://bitcoin-rpc.publicnode.com")

    mock_response = MagicMock()
    mock_response.json.return_value = {"result": "0" * 64, "error": None, "id": 1}
    mock_response.status_code = 200

    with patch.object(rpc._client, "post", new_callable=AsyncMock) as mock_post:
        mock_post.return_value = mock_response

        result = await rpc.get_block_hash(123456)
        assert result == "0" * 64


@pytest.mark.asyncio
async def test_get_block_header() -> None:
    rpc = BitcoinRPC(url="https://bitcoin-rpc.publicnode.com")

    expected_response = {
        "hash": "abc123",
        "confirmations": 10,
        "height": 123456,
        "version": 1,
        "versionHex": "00000001",
        "merkleroot": "def456",
        "time": 1234567890,
        "mediantime": 1234567800,
        "nonce": 12345,
        "bits": "1d00ffff",
        "difficulty": 1.0,
        "chainwork": "00" * 32,
        "nTx": 5,
    }

    mock_response = MagicMock()
    mock_response.json.return_value = {"result": expected_response, "error": None, "id": 1}
    mock_response.status_code = 200

    with patch.object(rpc._client, "post", new_callable=AsyncMock) as mock_post:
        mock_post.return_value = mock_response

        result = await rpc.get_block_header("fake_block_hash")
        assert result.hash == "abc123"
        assert result.height == 123456
        assert result.merkleroot == "def456"


@pytest.mark.asyncio
async def test_rpc_context_manager() -> None:
    async with BitcoinRPC(url="https://bitcoin-rpc.publicnode.com") as rpc:
        assert rpc is not None
