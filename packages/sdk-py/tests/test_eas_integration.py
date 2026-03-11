"""Integration tests for EAS (Ethereum Attestation Service).

These tests make real network calls to Ethereum RPC endpoints.
Set UTS_ETH_RPC_URL_11155111 environment variable to run them.
"""

from __future__ import annotations

import os

import pytest

from uts_sdk._ethereum.eas import EAS_SCHEMA_ID, EasContract, OnChainAttestation

# Skip all tests in this module if no Sepolia RPC is configured
pytestmark = pytest.mark.skipif(
    not os.environ.get("UTS_ETH_RPC_URL_11155111"),
    reason="UTS_ETH_RPC_URL_11155111 not set",
)

SEPOLIA_RPC = os.environ.get("UTS_ETH_RPC_URL_11155111", "")
SEPOLIA_EAS = "0xC2679fBD37d54388Ce493F1DB75320D236e1815e"


@pytest.fixture
async def eas_contract() -> EasContract:
    """Create an EasContract instance connected to Sepolia."""
    from web3 import AsyncWeb3

    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider(SEPOLIA_RPC))
    return EasContract(w3, SEPOLIA_EAS)  # type: ignore[arg-type]


class TestEasContractIntegration:
    """Real integration tests for EAS contract on Sepolia."""

    @pytest.mark.asyncio
    async def test_get_timestamp_known_data(self, eas_contract: EasContract) -> None:
        """Test getTimestamp with known timestamped data."""
        known_data = bytes.fromhex(
            "D445CE83D2BC148E8DDDDBC0EC6602D29A5ACAC302CE3E222833D4E830976381"
        )

        result = await eas_contract.get_timestamp(known_data)

        assert result == 1773009984

    @pytest.mark.asyncio
    async def test_get_timestamp_pads_short_data(
        self, eas_contract: EasContract
    ) -> None:
        """Test that short data is properly padded to 32 bytes."""
        short_data = b"\x01\x02\x03"

        result = await eas_contract.get_timestamp(short_data)

        assert isinstance(result, int)
        assert result >= 0

    @pytest.mark.asyncio
    async def test_get_attestation_known_uid(self, eas_contract: EasContract) -> None:
        """Test getAttestation with known attestation UID."""
        known_uid = bytes.fromhex(
            "09b4943032820e002fdd690783e3e76e3c8af7df0f8e56fa953317dac8f1f5a8"
        )

        result = await eas_contract.get_attestation(known_uid)

        assert isinstance(result, OnChainAttestation)
        assert result.data.startswith("0xcee")

    @pytest.mark.asyncio
    async def test_get_attestation_invalid_uid_raises(
        self, eas_contract: EasContract
    ) -> None:
        """Test that getAttestation raises for invalid UID length."""
        invalid_uid = b"\x01\x02\x03"

        with pytest.raises(ValueError, match="UID must be 32 bytes"):
            await eas_contract.get_attestation(invalid_uid)

    @pytest.mark.asyncio
    async def test_eas_schema_id_constant(self) -> None:
        """Verify EAS_SCHEMA_ID is the expected value."""
        assert (
            EAS_SCHEMA_ID
            == "0x5c5b8b295ff43c8e442be11d569e94a4cd5476f5e23df0f71bdd408df6b9649c"
        )
