// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import {Test, console} from "forge-std/Test.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {UniversalTimestamps} from "../contracts/core/UniversalTimestamps.sol";
import {L1FeeOracle} from "../contracts/L2/oracle/L1FeeOracle.sol";

/**
 * @title UniversalTimestampsTest
 * @dev Tests to verify the functionality of the UniversalTimestamps contract.
 */
contract UniversalTimestampsTest is Test {
    UniversalTimestamps uts;

    function setUp() public {
        uts = new UniversalTimestamps();
        console.log("UniversalTimestamps deployed at:", address(uts));
    }

    function test_AttestGasCost() public {
        bytes32 data = keccak256("Test data for attestation");
        uint256 startGas = gasleft();
        uts.attest(data);
        uint256 endGas = gasleft();
        // Adding base transaction cost
        uint256 gasUsed = startGas - endGas + 21000;
        console.log("Gas used for attest:", gasUsed);

        // Assert the L1FeeOracle default fee is roughly same.
        L1FeeOracle feeOracle = new L1FeeOracle(address(this));
        uint256 gas = feeOracle.gasPerAttestation();
        console.log("Current gas per attestation from L1FeeOracle:", gas);
        assertApproxEqAbs(gasUsed, gas, 5000); // Allow a margin of error of 5000 gas units
    }
}
