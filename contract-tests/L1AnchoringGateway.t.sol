// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

import {Test} from "forge-std/Test.sol";
import {L1AnchoringGateway} from "../contracts/L1/L1AnchoringGateway.sol";
import {IL1AnchoringGateway} from "../contracts/L1/IL1AnchoringGateway.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {IEAS} from "eas-contracts/IEAS.sol";
import {TestEASHelper} from "./EAS.t.sol";

contract MockL1ScrollMessenger {
    struct Message {
        address to;
        uint256 value;
        bytes message;
        uint256 gasLimit;
        address refundAddress;
    }

    Message public lastMessage;

    function sendMessage(address to, uint256 value, bytes calldata message, uint256 gasLimit, address refundAddress)
        external
        payable
    {
        lastMessage = Message(to, value, message, gasLimit, refundAddress);
    }
}

contract L1AnchoringGatewayTest is Test {
    L1AnchoringGateway gateway;
    IEAS eas;
    MockL1ScrollMessenger mockMessenger;

    address admin = address(this);
    address newAdmin = address(0xAD);
    address submitter = address(0x5B);
    address l2Manager = address(0x1212);

    function setUp() public {
        eas = TestEASHelper(vm.deployCode("TestEASHelper")).eas();
        mockMessenger = new MockL1ScrollMessenger();

        L1AnchoringGateway impl = L1AnchoringGateway(payable(vm.deployCode("L1AnchoringGateway")));
        address proxy = address(new ERC1967Proxy(address(impl), abi.encodeCall(L1AnchoringGateway.initialize, (admin))));
        gateway = L1AnchoringGateway(payable(proxy));

        vm.mockCall(address(mockMessenger), abi.encodeWithSignature("rollup()"), abi.encode(address(0)));
        gateway.lateInitialize(newAdmin, address(eas), address(mockMessenger), l2Manager);
        vm.warp(block.timestamp + 1);

        vm.startPrank(newAdmin);
        gateway.completeInitialization();
        // Grant submitter role (admin is now newAdmin)
        gateway.grantRole(gateway.SUBMITTER_ROLE(), submitter);
        vm.stopPrank();
    }

    function test_Initialize() public view {
        assertTrue(gateway.hasRole(gateway.DEFAULT_ADMIN_ROLE(), newAdmin));
        assertTrue(gateway.hasRole(gateway.SUBMITTER_ROLE(), submitter));
        assertEq(gateway.MAX_BATCH_SIZE(), 512);
        assertEq(gateway.MIN_GAS_LIMIT(), 110_000);
        assertEq(gateway.MAX_GAS_LIMIT(), 200_000);
    }

    function test_LateInitialize() public {
        // Deploy a fresh gateway
        L1AnchoringGateway impl = L1AnchoringGateway(payable(vm.deployCode("L1AnchoringGateway")));
        address proxy = address(new ERC1967Proxy(address(impl), abi.encodeCall(L1AnchoringGateway.initialize, (admin))));
        L1AnchoringGateway gw = L1AnchoringGateway(payable(proxy));

        address a = address(0xA1);
        gw.lateInitialize(a, address(eas), address(mockMessenger), l2Manager);

        vm.warp(block.timestamp + 1);
        vm.prank(a);
        gw.completeInitialization();

        assertTrue(gw.hasRole(gw.DEFAULT_ADMIN_ROLE(), a));
    }

    function test_LateInitialize_RevertZeroAddresses() public {
        L1AnchoringGateway impl = L1AnchoringGateway(payable(vm.deployCode("L1AnchoringGateway")));

        // Zero admin
        address proxy = address(new ERC1967Proxy(address(impl), abi.encodeCall(L1AnchoringGateway.initialize, (admin))));
        L1AnchoringGateway gw = L1AnchoringGateway(payable(proxy));
        vm.expectRevert(L1AnchoringGateway.InvalidAddress.selector);
        gw.lateInitialize(address(0), address(eas), address(mockMessenger), l2Manager);

        // Zero EAS
        vm.expectRevert(L1AnchoringGateway.InvalidAddress.selector);
        gw.lateInitialize(newAdmin, address(0), address(mockMessenger), l2Manager);

        // Zero messenger
        vm.expectRevert(L1AnchoringGateway.InvalidAddress.selector);
        gw.lateInitialize(newAdmin, address(eas), address(0), l2Manager);

        // Zero manager
        vm.expectRevert(L1AnchoringGateway.InvalidAddress.selector);
        gw.lateInitialize(newAdmin, address(eas), address(mockMessenger), address(0));
    }

    function test_SubmitBatch_WithNewTimestamp() public {
        bytes32 root = keccak256("new-root");
        // root not yet timestamped in EAS → getTimestamp returns 0 → gateway calls eas.timestamp()

        vm.deal(submitter, 1 ether);
        vm.prank(submitter);
        gateway.submitBatch{value: 0.1 ether}(root, 1, 10, 150_000);

        (address to,,, uint256 gasLimit,) = mockMessenger.lastMessage();
        assertEq(to, l2Manager);
        assertEq(gasLimit, 150_000);
    }

    function test_SubmitBatch_WithExistingTimestamp() public {
        bytes32 root = keccak256("pre-timestamped");
        // Pre-timestamp the root via EAS so getTimestamp returns non-zero
        eas.timestamp(root);

        vm.deal(submitter, 1 ether);
        vm.prank(submitter);
        gateway.submitBatch{value: 0.1 ether}(root, 1, 5, 150_000);

        (address to,,,,) = mockMessenger.lastMessage();
        assertEq(to, l2Manager);
    }

    function test_SubmitBatch_RevertNoSubmitterRole() public {
        bytes32 root = keccak256("test");
        address nonSubmitter = address(0x999);

        vm.deal(nonSubmitter, 1 ether);
        vm.prank(nonSubmitter);
        vm.expectRevert();
        gateway.submitBatch{value: 0.1 ether}(root, 1, 10, 150_000);
    }

    function test_SubmitBatch_RevertInvalidBatchSize() public {
        bytes32 root = keccak256("test");
        vm.deal(submitter, 1 ether);

        // count = 0
        vm.prank(submitter);
        vm.expectRevert(L1AnchoringGateway.InvalidBatchSize.selector);
        gateway.submitBatch{value: 0.1 ether}(root, 1, 0, 150_000);

        // count > 512
        vm.prank(submitter);
        vm.expectRevert(L1AnchoringGateway.InvalidBatchSize.selector);
        gateway.submitBatch{value: 0.1 ether}(root, 1, 513, 150_000);
    }

    function test_SubmitBatch_RevertInvalidGasLimit() public {
        bytes32 root = keccak256("test");
        vm.deal(submitter, 1 ether);

        // too low
        vm.prank(submitter);
        vm.expectRevert(L1AnchoringGateway.InvalidGasLimit.selector);
        gateway.submitBatch{value: 0.1 ether}(root, 1, 10, 109_999);

        // too high
        vm.prank(submitter);
        vm.expectRevert(L1AnchoringGateway.InvalidGasLimit.selector);
        gateway.submitBatch{value: 0.1 ether}(root, 1, 10, 200_001);
    }

    function test_SubmitBatch_RevertMessengerNotSet() public {
        L1AnchoringGateway impl = L1AnchoringGateway(payable(vm.deployCode("L1AnchoringGateway")));
        address proxy = address(new ERC1967Proxy(address(impl), abi.encodeCall(L1AnchoringGateway.initialize, (admin))));
        L1AnchoringGateway gw = L1AnchoringGateway(payable(proxy));
        gw.grantRole(gw.SUBMITTER_ROLE(), submitter);

        vm.deal(submitter, 1 ether);
        vm.prank(submitter);
        vm.expectRevert(L1AnchoringGateway.InvalidAddress.selector);
        gw.submitBatch{value: 0.1 ether}(keccak256("test"), 1, 10, 150_000);
    }

    function test_SubmitBatch_RevertManagerNotSet() public {
        L1AnchoringGateway impl = L1AnchoringGateway(payable(vm.deployCode("L1AnchoringGateway")));
        address proxy = address(new ERC1967Proxy(address(impl), abi.encodeCall(L1AnchoringGateway.initialize, (admin))));
        L1AnchoringGateway gw = L1AnchoringGateway(payable(proxy));
        gw.grantRole(gw.SUBMITTER_ROLE(), submitter);
        gw.setL1ScrollMessenger(address(mockMessenger));

        vm.deal(submitter, 1 ether);
        vm.prank(submitter);
        vm.expectRevert(L1AnchoringGateway.InvalidAddress.selector);
        gw.submitBatch{value: 0.1 ether}(keccak256("test"), 1, 10, 150_000);
    }

    function test_SetL1ScrollMessenger() public {
        address newMessenger = address(0xBEEF);
        vm.prank(newAdmin);
        gateway.setL1ScrollMessenger(newMessenger);
    }

    function test_SetL1ScrollMessenger_RevertZeroAddress() public {
        vm.prank(newAdmin);
        vm.expectRevert(L1AnchoringGateway.InvalidAddress.selector);
        gateway.setL1ScrollMessenger(address(0));
    }

    function test_SetL1ScrollMessenger_RevertNotAdmin() public {
        vm.prank(address(0xBAD));
        vm.expectRevert();
        gateway.setL1ScrollMessenger(address(0xBEEF));
    }

    function test_SetL2AnchoringManager() public {
        address newManager = address(0xCAFE);
        vm.prank(newAdmin);
        gateway.setL2AnchoringManager(newManager);
    }

    function test_SetL2AnchoringManager_RevertZeroAddress() public {
        vm.prank(newAdmin);
        vm.expectRevert(L1AnchoringGateway.InvalidAddress.selector);
        gateway.setL2AnchoringManager(address(0));
    }

    function test_SetL2AnchoringManager_RevertNotAdmin() public {
        vm.prank(address(0xBAD));
        vm.expectRevert();
        gateway.setL2AnchoringManager(address(0xCAFE));
    }

    function test_SubmitBatch_EmitsBatchSubmittedEvent() public {
        bytes32 root = keccak256("event-test");

        vm.deal(submitter, 1 ether);
        vm.prank(submitter);
        vm.expectEmit(true, true, true, true);
        emit IL1AnchoringGateway.BatchSubmitted(root, 1, 10, submitter);
        gateway.submitBatch{value: 0.1 ether}(root, 1, 10, 150_000);
    }
}
