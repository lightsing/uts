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

    function test_Basic() public {
        bytes32 root = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
        uint256 fee = feeOracle.getFloorFee();
        console.log("Current floor fee:", fee);

        // Simulate submitting a root for L1 anchoring
        vm.prank(address(1)); // Simulate a call from an external address
        vm.deal(address(1), 100 ether); // Fund the address with some ether to pay for the fee
        vm.expectEmit(true, true, true, true);
        emit IL2AnchoringManager.L1AnchoringQueued(root, 1, fee, block.number, block.timestamp);
        manager.submitForL1Anchoring{value: fee}(root, address(1));

        // Verify that the item was added to the queue
        bool confirmed = manager.isConfirmed(root);
        assertFalse(confirmed, "Root should not be confirmed immediately after submission");

        // Simulate a call from bridge to confirm the anchoring
        vm.prank(address(l2Messenger)); // Simulate a call from the L2 messenger
        vm.expectEmit(true, true, true, true);
        emit IL2AnchoringManager.L1BatchArrived(root, 1, 1, block.number, block.number, block.timestamp);
        manager.notifyAnchored(root, 1, 1, block.number);

        vm.prank(address(1));
        vm.expectEmit(true, true, true, true);
        emit IL2AnchoringManager.L1BatchFinalized(root, 1, 1, block.number, block.number, block.timestamp);
        manager.finalizeBatch();
    }

    function test_NonExistentRoot() public view {
        bytes32 root = 0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd;
        bool confirmed = manager.isConfirmed(root);
        assertFalse(confirmed, "Non-existent root should not be confirmed");
    }
}

contract L2AnchoringManagerGasTest is Test {
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

        // fill 1024 roots
        vm.deal(address(1), 100 ether);
        uint256 fee = feeOracle.getFloorFee();
        for (uint256 i = 0; i < 1024; i++) {
            bytes32 root = keccak256(abi.encodePacked(i));
            vm.prank(address(1));
            manager.submitForL1Anchoring{value: fee}(root, address(1));
        }
    }

    function confirmBatchGas(bytes32 expectedRoot, uint256 startIndex, uint256 count) private {
        vm.prank(address(l2Messenger));
        uint256 startGas = gasleft();
        manager.notifyAnchored(expectedRoot, startIndex, count, block.number);
        uint256 l1L2GasUsed = startGas - gasleft();

        vm.prank(address(1));
        startGas = gasleft();
        manager.finalizeBatch();
        uint256 l2GasUsed = startGas - gasleft();

        console.log("Gas used for confirming batch with count", count, l1L2GasUsed, l2GasUsed);
    }

    function test_ConfirmBatchGas_1() public {
        bytes32 expectedRoot = bytes32(0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);
        confirmBatchGas(expectedRoot, 1, 1);
    }

    function test_ConfirmBatchGas_2() public {
        bytes32 expectedRoot = bytes32(0x05cca086ac1292d712fa72c1b3f12ca115644d2961f946ba815d8d731f9e5059);
        confirmBatchGas(expectedRoot, 1, 2);
    }

    function test_ConfirmBatchGas_4() public {
        bytes32 expectedRoot = bytes32(0x0a9951a6344d06e27cd299dad49803dfb69d0009bca6dd3aa073a6e9dcfd7aa7);
        confirmBatchGas(expectedRoot, 1, 4);
    }

    function test_ConfirmBatchGas_8() public {
        bytes32 expectedRoot = bytes32(0xc1df1e1138de013d39e21ddf1b1ab0dd91028c8d002c83be7b1fcdfc68035b6d);
        confirmBatchGas(expectedRoot, 1, 8);
    }

    function test_ConfirmBatchGas_16() public {
        bytes32 expectedRoot = bytes32(0xf1c740406f2ba80a76a186c2c0e76282812958712624855b1c9411dfc9a6792c);
        confirmBatchGas(expectedRoot, 1, 16);
    }

    function test_ConfirmBatchGas_32() public {
        bytes32 expectedRoot = bytes32(0xee50fb68d594b2edce57c20f37f15319dbe5726b7a8ca77397e0ea34222460f3);
        confirmBatchGas(expectedRoot, 1, 32);
    }

    function test_ConfirmBatchGas_64() public {
        bytes32 expectedRoot = bytes32(0xf74fe0dade4345ea10ff784b4ff5989fc98d352bc2c771c75532915a3fe4088c);
        confirmBatchGas(expectedRoot, 1, 64);
    }

    function test_ConfirmBatchGas_128() public {
        bytes32 expectedRoot = bytes32(0x679299f8934b50bb98d4801909db25cb0328bc553ecf89be4203e4268822c892);
        confirmBatchGas(expectedRoot, 1, 128);
    }

    function test_ConfirmBatchGas_256() public {
        bytes32 expectedRoot = bytes32(0x21ba7c62f00c063ebb82e15aa3f706dc15371e1716390f820adc4a15d58358de);
        confirmBatchGas(expectedRoot, 1, 256);
    }

    function test_ConfirmBatchGas_512() public {
        bytes32 expectedRoot = bytes32(0xc8f2d024989f2ec0438755b10bed2cd22998585c2a87f16661b73ea62e604d4e);
        confirmBatchGas(expectedRoot, 1, 512);
    }

    function test_ConfirmBatchGas_1024() public {
        bytes32 expectedRoot = bytes32(0x168852131b9462b8137b1deadf05872035b6046281ecf234278545aef52ae47b);
        confirmBatchGas(expectedRoot, 1, 1024);
    }
}
