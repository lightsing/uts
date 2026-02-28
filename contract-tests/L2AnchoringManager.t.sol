// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import {Test, console} from "forge-std/Test.sol";
import {L2AnchoringManager} from "../contracts/L2/manager/L2AnchoringManager.sol";
import {IL2AnchoringManager} from "../contracts/L2/manager/IL2AnchoringManager.sol";
import {IL1FeeOracle} from "../contracts/L2/oracle/IL1FeeOracle.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {UniversalTimestamps} from "../contracts/core/UniversalTimestamps.sol";
import {IUniversalTimestamps} from "../contracts/core/IUniversalTimestamps.sol";
import {IL2ScrollMessenger} from "scroll-contracts/L2/IL2ScrollMessenger.sol";

contract MockL1FeeOracle is IL1FeeOracle {
    function getL1BaseFee() external view returns (uint256) {
        return 0.05 gwei;
    }

    function getFeePerAttestation() external view returns (uint256) {
        return 0.05 gwei * 51_000;
    }

    function getFloorFee() external view returns (uint256) {
        return (0.05 gwei * 51_000 * 0.5e18) / 1e18;
    }

    function getGasPerAttestation() external view returns (uint256) {
        return 51_000;
    }

    function getDiscountRatio() external view returns (uint256) {
        return 0.5e18; // 50% discount
    }
}

contract MockL2ScrollMessenger is IL2ScrollMessenger {
    function relayMessage(address from, address to, uint256 value, uint256 nonce, bytes calldata message) external {}

    function xDomainMessageSender() external view returns (address) {
        return address(0x456); // Mock L1 sender address
    }

    function sendMessage(address target, uint256 value, bytes calldata message, uint256 gasLimit) external payable {}

    function sendMessage(address target, uint256 value, bytes calldata message, uint256 gasLimit, address refundAddress)
        external
        payable {}
}

/**
 * @title L2AnchoringManagerTest
 * @dev Tests to verify the functionality of the L2AnchoringManager contract.
 */
contract L2AnchoringManagerTest is Test {
    IUniversalTimestamps uts;
    IL1FeeOracle feeOracle;
    IL2AnchoringManager manager;
    IL2ScrollMessenger l2Messenger;

    address constant L1_GATEWAY = address(0x456);

    function setUp() public {
        uts = new UniversalTimestamps();
        feeOracle = new MockL1FeeOracle();
        l2Messenger = new MockL2ScrollMessenger();

        L2AnchoringManager impl = new L2AnchoringManager();
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(impl),
            abi.encodeCall(
                L2AnchoringManager.initialize, (address(this), address(uts), address(feeOracle), address(l2Messenger))
            )
        );
        manager = IL2AnchoringManager(address(proxy));
        manager.setL1Gateway(L1_GATEWAY);
    }

    function test() public {
        bytes32 root = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
        uint256 fee = feeOracle.getFloorFee();
        console.log("Current floor fee:", fee);

        // Simulate submitting a root for L1 anchoring
        vm.prank(address(1)); // Simulate a call from an external address
        vm.deal(address(1), 100 ether); // Fund the address with some ether to pay for the fee
        vm.expectEmit(true, true, true, true);
        emit IL2AnchoringManager.L1AnchoringQueued(root, 0, fee, block.number, block.timestamp);
        manager.submitForL1Anchoring{value: fee}(root, address(1));

        // Verify that the item was added to the queue
        bool confirmed = manager.isConfirmed(root);
        assertFalse(confirmed, "Root should not be confirmed immediately after submission");

        // Simulate a call from bridge to confirm the anchoring
        vm.prank(address(l2Messenger)); // Simulate a call from the L2 messenger
        vm.expectEmit(true, true, true, true);
        emit IL2AnchoringManager.L1AnchoringBatchConfirmed(root, 0, 1, block.number, block.number, block.timestamp);
        manager.confirmL1AnchoringBatch(root, 0, 1, block.number);
    }
}
