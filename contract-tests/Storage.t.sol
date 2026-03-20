// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {SlotDerivation} from "@openzeppelin/contracts/utils/SlotDerivation.sol";
import {L1AnchoringGatewayStorage} from "../contracts/L1/L1AnchoringGatewayStorage.sol";
import {L2AnchoringManagerStorage} from "../contracts/L2/manager/L2AnchoringManagerStorage.sol";

/**
 * @title UniversalTimestampsStorageTest
 * @dev Tests to verify the integrity of the storage slot derivation and layout.
 */
contract StorageSlotTest is Test {
    using SlotDerivation for string;

    function test_ERC7201_SlotDerivation() public pure {
        test(L1AnchoringGatewayStorage.SLOT, L1AnchoringGatewayStorage.NAMESPACE);

        test(L2AnchoringManagerStorage.SLOT, L2AnchoringManagerStorage.NAMESPACE);
    }

    function test(bytes32 hardcodedSlot, string memory namespace) internal pure virtual {
        bytes32 expectedSlot = namespace.erc7201Slot();
        assertEq(
            hardcodedSlot,
            expectedSlot,
            "Storage SLOT mismatch: The hardcoded slot does not match the ERC-7201 derivation of the namespace."
        );

        console.log("Namespace:", namespace);
        console.log("Hardcoded Slot:", vm.toString(hardcodedSlot));
        console.log("Expected Slot: ", vm.toString(expectedSlot));
    }
}
