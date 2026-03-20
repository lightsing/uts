"""Tests for EAS (Ethereum Attestation Service) integration."""

from __future__ import annotations

from dataclasses import FrozenInstanceError
from unittest.mock import AsyncMock, MagicMock

import pytest

from uts_sdk._ethereum.eas import EasContract, OnChainAttestation


@pytest.fixture
def mock_web3() -> MagicMock:
    """Create a mock AsyncWeb3 instance."""
    w3 = MagicMock()
    w3.eth = MagicMock()
    return w3


@pytest.fixture
def mock_contract() -> MagicMock:
    """Create a mock EAS contract."""
    contract = MagicMock()
    contract.functions = MagicMock()
    return contract


class TestOnChainAttestation:
    """Tests for OnChainAttestation dataclass."""

    def test_on_chain_attestation_creation(self) -> None:
        att = OnChainAttestation(
            uid="0x" + "00" * 32,
            schema="0x" + "aa" * 32,
            time=1234567890,
            expiration_time=0,
            revocation_time=0,
            ref_uid="0x" + "00" * 32,
            recipient="0x" + "bb" * 20,
            attester="0x" + "cc" * 20,
            revocable=False,
            data="0x" + "dd" * 32,
        )

        assert att.uid.startswith("0x")
        assert att.time == 1234567890
        assert att.expiration_time == 0
        assert not att.revocable

    def test_on_chain_attestation_frozen(self) -> None:
        att = OnChainAttestation(
            uid="0x" + "00" * 32,
            schema="0x" + "aa" * 32,
            time=100,
            expiration_time=0,
            revocation_time=0,
            ref_uid="0x" + "00" * 32,
            recipient="0x" + "bb" * 20,
            attester="0x" + "cc" * 20,
            revocable=False,
            data="0x",
        )

        with pytest.raises(FrozenInstanceError):
            att.time = 200  # type: ignore[misc]


class TestEasContract:
    """Tests for EasContract class."""

    def test_eas_contract_initialization(
        self, mock_web3: MagicMock, mock_contract: MagicMock
    ) -> None:
        mock_web3.eth.contract = MagicMock(return_value=mock_contract)

        eas = EasContract(mock_web3, "0xA1207F3BBa224E2c9c3c6D5aF63D0eb1582Ce587")  # type: ignore[arg-type]

        assert eas._w3 == mock_web3
        assert eas._contract == mock_contract

    @pytest.mark.asyncio
    async def test_get_timestamp(
        self, mock_web3: MagicMock, mock_contract: MagicMock
    ) -> None:
        mock_get_timestamp = AsyncMock(return_value=1234567890)
        mock_contract.functions.getTimestamp = MagicMock(
            return_value=MagicMock(call=mock_get_timestamp)
        )
        mock_web3.eth.contract = MagicMock(return_value=mock_contract)

        eas = EasContract(mock_web3, "0xA1207F3BBa224E2c9c3c6D5aF63D0eb1582Ce587")  # type: ignore[arg-type]
        data = b"\x00" * 32

        result = await eas.get_timestamp(data)

        assert result == 1234567890
        mock_get_timestamp.assert_called_once()

    @pytest.mark.asyncio
    async def test_get_timestamp_pads_short_data(
        self, mock_web3: MagicMock, mock_contract: MagicMock
    ) -> None:
        called_data: bytes = b""
        mock_get_timestamp = AsyncMock(return_value=0)

        def capture_padded(data: bytes) -> MagicMock:
            nonlocal called_data
            called_data = data
            return MagicMock(call=mock_get_timestamp)

        mock_contract.functions.getTimestamp = capture_padded
        mock_web3.eth.contract = MagicMock(return_value=mock_contract)

        eas = EasContract(mock_web3, "0xA1207F3BBa224E2c9c3c6D5aF63D0eb1582Ce587")  # type: ignore[arg-type]
        short_data = b"\x01\x02\x03"

        await eas.get_timestamp(short_data)

        assert len(called_data) == 32
        assert called_data[:3] == b"\x01\x02\x03"
        assert called_data[3:] == b"\x00" * 29

    @pytest.mark.asyncio
    async def test_get_timestamp_truncates_long_data(
        self, mock_web3: MagicMock, mock_contract: MagicMock
    ) -> None:
        called_data: bytes = b""
        mock_get_timestamp = AsyncMock(return_value=0)

        def capture_padded(data: bytes) -> MagicMock:
            nonlocal called_data
            called_data = data
            return MagicMock(call=mock_get_timestamp)

        mock_contract.functions.getTimestamp = capture_padded
        mock_web3.eth.contract = MagicMock(return_value=mock_contract)

        eas = EasContract(mock_web3, "0xA1207F3BBa224E2c9c3c6D5aF63D0eb1582Ce587")  # type: ignore[arg-type]
        long_data = b"\xff" * 64

        await eas.get_timestamp(long_data)

        assert len(called_data) == 32
        assert called_data == b"\xff" * 32

    @pytest.mark.asyncio
    async def test_get_attestation(
        self, mock_web3: MagicMock, mock_contract: MagicMock
    ) -> None:
        uid = b"\x01" * 32
        mock_result = (
            uid,  # uid
            b"\x02" * 32,  # schema
            1234567890,  # time
            0,  # expiration_time
            0,  # revocation_time
            b"\x00" * 32,  # ref_uid
            "0xRecipientAddress",  # recipient
            "0xAttesterAddress",  # attester
            False,  # revocable
            b"\x03" * 32,  # data
        )
        mock_get_attestation = AsyncMock(return_value=mock_result)
        mock_contract.functions.getAttestation = MagicMock(
            return_value=MagicMock(call=mock_get_attestation)
        )
        mock_web3.eth.contract = MagicMock(return_value=mock_contract)

        eas = EasContract(mock_web3, "0xA1207F3BBa224E2c9c3c6D5aF63D0eb1582Ce587")  # type: ignore[arg-type]

        result = await eas.get_attestation(uid)

        assert isinstance(result, OnChainAttestation)
        assert result.time == 1234567890
        assert not result.revocable

    @pytest.mark.asyncio
    async def test_get_attestation_invalid_uid_length(
        self, mock_web3: MagicMock, mock_contract: MagicMock
    ) -> None:
        mock_web3.eth.contract = MagicMock(return_value=mock_contract)
        eas = EasContract(mock_web3, "0xA1207F3BBa224E2c9c3c6D5aF63D0eb1582Ce587")  # type: ignore[arg-type]

        with pytest.raises(ValueError, match="UID must be 32 bytes"):
            await eas.get_attestation(b"\x01\x02\x03")
