// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

import {Test, console, Vm} from "forge-std/Test.sol";
import {L2AnchoringManager} from "../contracts/L2/manager/L2AnchoringManager.sol";
import {IL2AnchoringManager} from "../contracts/L2/manager/IL2AnchoringManager.sol";
import {IFeeOracle} from "../contracts/L2/oracle/IFeeOracle.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {IL2ScrollMessenger} from "scroll-contracts/L2/IL2ScrollMessenger.sol";
import {ScrollConstants} from "scroll-contracts/libraries/constants/ScrollConstants.sol";
import {TestEASHelper} from "./EAS.t.sol";
import {IEAS} from "eas-contracts/IEAS.sol";
import {INFTGenerator} from "../contracts/L2/nft/INFTGenerator.sol";
import {MerkleTree} from "../contracts/core/MerkleTree.sol";
import {MockFeeOracle, MockL2ScrollMessenger} from "./L2AnchoringManager.t.sol";

contract MockNFTGenerator is INFTGenerator {
    function generateTokenURI(uint256, bytes32, uint256, uint256, uint256, string memory)
        external
        pure
        override
        returns (string memory)
    {
        return "mock://token";
    }
}

contract L2AnchoringManagerExtendedTest is Test {
    IEAS eas;
    MockFeeOracle feeOracle;
    L2AnchoringManager managerInstance;
    IL2AnchoringManager manager;
    MockL2ScrollMessenger l2Messenger;
    INFTGenerator nftGenerator;

    address constant L1_GATEWAY = address(0x123);
    address user = address(0x1);

    function setUp() public {
        eas = TestEASHelper(vm.deployCode("TestEASHelper")).eas();
        feeOracle = new MockFeeOracle();
        l2Messenger = new MockL2ScrollMessenger();
        nftGenerator = new MockNFTGenerator();

        L2AnchoringManager impl = L2AnchoringManager(payable(vm.deployCode("L2AnchoringManager")));
        address proxy = address(new ERC1967Proxy(address(impl), abi.encodeCall(L2AnchoringManager.initialize, ())));
        managerInstance = L2AnchoringManager(payable(proxy));
        managerInstance.lateInitialize(
            "Scroll", address(this), address(eas), address(feeOracle), address(l2Messenger), address(nftGenerator)
        );
        vm.warp(block.timestamp + 1);
        managerInstance.completeInitialization();

        manager = IL2AnchoringManager(address(managerInstance));
        manager.setL1Gateway(L1_GATEWAY);

        vm.deal(user, 100 ether);
    }

    // --- submitForL1Anchoring ---

    function test_SubmitForL1Anchoring_RevertInsufficientFee() public {
        bytes32 root = keccak256("root1");
        uint256 fee = feeOracle.getFloorFee();

        vm.prank(user);
        vm.expectRevert(L2AnchoringManager.InsufficientFee.selector);
        manager.submitForL1Anchoring{value: fee - 1}(root);
    }

    function test_SubmitForL1Anchoring_RefundsExcess() public {
        bytes32 root = keccak256("root2");
        uint256 fee = feeOracle.getFloorFee();
        uint256 excess = 1 ether;

        uint256 balBefore = user.balance;
        vm.prank(user);
        manager.submitForL1Anchoring{value: fee + excess}(root, user);
        uint256 balAfter = user.balance;

        // User paid exactly fee (minus gas, but the refund should cover excess)
        assertEq(balBefore - balAfter, fee);
    }

    // --- notifyAnchored ---

    function test_NotifyAnchored_RevertInvalidBatchCount() public {
        l2Messenger.setSender(L1_GATEWAY);
        vm.prank(address(l2Messenger));
        vm.expectRevert(L2AnchoringManager.InvalidBatchCount.selector);
        manager.notifyAnchored(keccak256("root"), 1, 0, block.timestamp, block.number);
    }

    function test_NotifyAnchored_RevertInvalidL2Messenger() public {
        vm.prank(address(0xBAD));
        vm.expectRevert(L2AnchoringManager.InvalidL2Messenger.selector);
        manager.notifyAnchored(keccak256("root"), 1, 1, block.timestamp, block.number);
    }

    function test_NotifyAnchored_RevertInvalidL1Sender() public {
        l2Messenger.setSender(address(0xBAD)); // not L1_GATEWAY
        vm.prank(address(l2Messenger));
        vm.expectRevert(L2AnchoringManager.InvalidL1Sender.selector);
        manager.notifyAnchored(keccak256("root"), 1, 1, block.timestamp, block.number);
    }

    function test_NotifyAnchored_RevertBatchAlreadyExists() public {
        bytes32 root = keccak256("root3");
        uint256 fee = feeOracle.getFloorFee();
        vm.prank(user);
        manager.submitForL1Anchoring{value: fee}(root);

        l2Messenger.setSender(L1_GATEWAY);
        vm.prank(address(l2Messenger));
        manager.notifyAnchored(root, 1, 1, block.timestamp, block.number);

        // Try to notify again without finalizing
        vm.prank(address(l2Messenger));
        vm.expectRevert(L2AnchoringManager.BatchAlreadyExists.selector);
        manager.notifyAnchored(root, 2, 1, block.timestamp, block.number);
    }

    function test_NotifyAnchored_RevertInvalidBatchOrder() public {
        bytes32 root = keccak256("root4");
        uint256 fee = feeOracle.getFloorFee();
        vm.prank(user);
        manager.submitForL1Anchoring{value: fee}(root);

        l2Messenger.setSender(L1_GATEWAY);
        vm.prank(address(l2Messenger));
        vm.expectRevert(L2AnchoringManager.InvalidBatchOrder.selector);
        manager.notifyAnchored(root, 999, 1, block.timestamp, block.number);
    }

    // --- finalizeBatch ---

    function test_FinalizeBatch_RevertNoPendingBatch() public {
        vm.expectRevert(L2AnchoringManager.NoPendingBatch.selector);
        manager.finalizeBatch();
    }

    function test_FinalizeBatch_RevertMerkleRootMismatch() public {
        bytes32 root = keccak256("root5");
        uint256 fee = feeOracle.getFloorFee();
        vm.prank(user);
        manager.submitForL1Anchoring{value: fee}(root);

        bytes32 wrongRoot = keccak256("wrong");
        l2Messenger.setSender(L1_GATEWAY);
        vm.prank(address(l2Messenger));
        manager.notifyAnchored(wrongRoot, 1, 1, block.timestamp, block.number);

        vm.expectRevert(L2AnchoringManager.MerkleRootMismatch.selector);
        manager.finalizeBatch();
    }

    // --- claimNFT full lifecycle ---

    function test_ClaimNFT_FullLifecycle() public {
        bytes32 root = keccak256("nft-root");
        uint256 fee = feeOracle.getFloorFee();

        // 1. Submit
        vm.prank(user);
        vm.recordLogs();
        manager.submitForL1Anchoring{value: fee}(root);

        // Extract attestationId from L1AnchoringQueued event
        Vm.Log[] memory logs = vm.getRecordedLogs();
        bytes32 attestationId;
        bytes32 eventSig = keccak256("L1AnchoringQueued(bytes32,bytes32,uint256,uint256,uint256,uint256)");
        for (uint256 i = 0; i < logs.length; i++) {
            if (logs[i].topics[0] == eventSig) {
                attestationId = logs[i].topics[1];
                break;
            }
        }
        assertTrue(attestationId != bytes32(0), "attestationId should be non-zero");

        // 2. Notify
        l2Messenger.setSender(L1_GATEWAY);
        vm.prank(address(l2Messenger));
        manager.notifyAnchored(root, 1, 1, block.timestamp, block.number);

        // 3. Finalize
        manager.finalizeBatch();
        assertTrue(manager.isConfirmed(root));

        // 4. Claim NFT - mock onERC721Received on the proxy (attester) so _safeMint succeeds
        vm.mockCall(
            address(managerInstance),
            abi.encodeWithSignature("onERC721Received(address,address,uint256,bytes)"),
            abi.encode(bytes4(keccak256("onERC721Received(address,address,uint256,bytes)")))
        );

        vm.prank(user);
        manager.claimNFT(attestationId, 1);
    }

    function test_ClaimNFT_RevertInvalidAttestationId() public {
        vm.expectRevert(L2AnchoringManager.InvalidAttestationId.selector);
        manager.claimNFT(keccak256("nonexistent"), 1);
    }

    function test_ClaimNFT_RevertNFTAlreadyClaimed() public {
        bytes32 root = keccak256("claim-twice");
        uint256 fee = feeOracle.getFloorFee();

        vm.prank(user);
        vm.recordLogs();
        manager.submitForL1Anchoring{value: fee}(root);
        Vm.Log[] memory logs = vm.getRecordedLogs();
        bytes32 attestationId;
        bytes32 eventSig = keccak256("L1AnchoringQueued(bytes32,bytes32,uint256,uint256,uint256,uint256)");
        for (uint256 i = 0; i < logs.length; i++) {
            if (logs[i].topics[0] == eventSig) {
                attestationId = logs[i].topics[1];
                break;
            }
        }

        l2Messenger.setSender(L1_GATEWAY);
        vm.prank(address(l2Messenger));
        manager.notifyAnchored(root, 1, 1, block.timestamp, block.number);
        manager.finalizeBatch();

        // Mock onERC721Received on the proxy (attester)
        vm.mockCall(
            address(managerInstance),
            abi.encodeWithSignature("onERC721Received(address,address,uint256,bytes)"),
            abi.encode(bytes4(keccak256("onERC721Received(address,address,uint256,bytes)")))
        );

        vm.prank(user);
        manager.claimNFT(attestationId, 1);

        vm.prank(user);
        vm.expectRevert(L2AnchoringManager.NFTAlreadyClaimed.selector);
        manager.claimNFT(attestationId, 1);
    }

    function test_ClaimNFT_RevertInvalidBatchIndexHint() public {
        bytes32 root = keccak256("hint-test");
        uint256 fee = feeOracle.getFloorFee();

        vm.prank(user);
        vm.recordLogs();
        manager.submitForL1Anchoring{value: fee}(root);
        Vm.Log[] memory logs = vm.getRecordedLogs();
        bytes32 attestationId;
        bytes32 eventSig = keccak256("L1AnchoringQueued(bytes32,bytes32,uint256,uint256,uint256,uint256)");
        for (uint256 i = 0; i < logs.length; i++) {
            if (logs[i].topics[0] == eventSig) {
                attestationId = logs[i].topics[1];
                break;
            }
        }

        l2Messenger.setSender(L1_GATEWAY);
        vm.prank(address(l2Messenger));
        manager.notifyAnchored(root, 1, 1, block.timestamp, block.number);
        manager.finalizeBatch();

        vm.prank(user);
        vm.expectRevert(L2AnchoringManager.InvalidBatchIndexHint.selector);
        manager.claimNFT(attestationId, 999);
    }

    // --- withdrawFees ---

    function test_WithdrawFees_RevertNotFeeCollector() public {
        vm.prank(address(0xBAD));
        vm.expectRevert();
        manager.withdrawFees(address(0xBAD), 1 ether);
    }

    function test_WithdrawFees_RevertInvalidAmount_Zero() public {
        managerInstance.grantRole(managerInstance.FEE_COLLECTOR_ROLE(), address(this));
        vm.expectRevert(L2AnchoringManager.InvalidAmount.selector);
        manager.withdrawFees(address(this), 0);
    }

    function test_WithdrawFees_RevertInvalidAmount_TooLarge() public {
        managerInstance.grantRole(managerInstance.FEE_COLLECTOR_ROLE(), address(this));
        vm.expectRevert(L2AnchoringManager.InvalidAmount.selector);
        manager.withdrawFees(address(this), 999 ether);
    }

    function test_WithdrawFees_Success() public {
        // Accumulate some fees
        bytes32 root = keccak256("fee-test");
        uint256 fee = feeOracle.getFloorFee();
        vm.prank(user);
        manager.submitForL1Anchoring{value: fee}(root);

        managerInstance.grantRole(managerInstance.FEE_COLLECTOR_ROLE(), address(this));

        address collector = address(0xC011);
        uint256 balance = address(managerInstance).balance;
        assertTrue(balance > 0, "Should have fees");

        manager.withdrawFees(collector, balance);
        assertEq(collector.balance, balance);
    }

    // --- clearBatch ---

    function test_ClearBatch_Success() public {
        bytes32 root = keccak256("clear-test");
        uint256 fee = feeOracle.getFloorFee();
        vm.prank(user);
        manager.submitForL1Anchoring{value: fee}(root);

        l2Messenger.setSender(L1_GATEWAY);
        vm.prank(address(l2Messenger));
        manager.notifyAnchored(root, 1, 1, block.timestamp, block.number);

        // Admin clears the batch
        manager.clearBatch();

        // Now finalize should fail since batch was cleared
        vm.expectRevert(L2AnchoringManager.NoPendingBatch.selector);
        manager.finalizeBatch();
    }

    function test_ClearBatch_RevertNotAdmin() public {
        vm.prank(address(0xBAD));
        vm.expectRevert();
        manager.clearBatch();
    }

    // --- setL2Messenger emits correct old address ---

    function test_SetL2Messenger_EmitsCorrectOldAddress() public {
        MockL2ScrollMessenger newMessenger = new MockL2ScrollMessenger();
        address oldMessenger = address(l2Messenger);

        vm.expectEmit(true, true, false, false);
        emit IL2AnchoringManager.L2MessengerUpdated(oldMessenger, address(newMessenger));
        manager.setL2Messenger(address(newMessenger));
    }

    // --- setFeeOracle / setL1Gateway revert on zero ---

    function test_SetFeeOracle_RevertZeroAddress() public {
        vm.expectRevert(L2AnchoringManager.InvalidAddress.selector);
        manager.setFeeOracle(address(0));
    }

    function test_SetL1Gateway_RevertZeroAddress() public {
        vm.expectRevert(L2AnchoringManager.InvalidAddress.selector);
        manager.setL1Gateway(address(0));
    }

    // --- lateInitialize revert already initialized ---

    function test_LateInitialize_RevertAlreadyInitialized() public {
        vm.expectRevert(L2AnchoringManager.AlreadyInitialized.selector);
        managerInstance.lateInitialize(
            "Scroll", address(this), address(eas), address(feeOracle), address(l2Messenger), address(nftGenerator)
        );
    }

    // --- isConfirmed returns false for unconfirmed ---

    function test_IsConfirmed_ReturnsFalseForUnconfirmed() public {
        bytes32 root = keccak256("unconfirmed");
        uint256 fee = feeOracle.getFloorFee();
        vm.prank(user);
        manager.submitForL1Anchoring{value: fee}(root);

        assertFalse(manager.isConfirmed(root));
    }
}
