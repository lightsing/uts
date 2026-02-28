// SPDX-License-Identifier: MIT

pragma solidity ^0.8.29;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {ReentrancyGuardTransient} from "@openzeppelin/contracts/utils/ReentrancyGuardTransient.sol";
import {IL2AnchoringManager} from "./IL2AnchoringManager.sol";
import {L2AnchoringManagerStorage} from "./L2AnchoringManagerStorage.sol";
import {IL1FeeOracle} from "../oracle/IL1FeeOracle.sol";
import {L2AnchoringManagerTypes} from "./L2AnchoringManagerTypes.sol";
import {MerkleTree} from "../../core/MerkleTree.sol";
import {IUniversalTimestamps} from "../../core/IUniversalTimestamps.sol";
import {IL2ScrollMessenger} from "scroll-contracts/L2/IL2ScrollMessenger.sol";
import {ScrollConstants} from "scroll-contracts/libraries/constants/ScrollConstants.sol";
import {
    AccessControlDefaultAdminRulesUpgradeable
} from "@openzeppelin/contracts-upgradeable/access/extensions/AccessControlDefaultAdminRulesUpgradeable.sol";

contract L2AnchoringManager is
    Initializable,
    UUPSUpgradeable,
    ReentrancyGuardTransient,
    AccessControlDefaultAdminRulesUpgradeable,
    IL2AnchoringManager
{
    bytes32 public constant FEE_COLLECTOR_ROLE = keccak256("FEE_COLLECTOR_ROLE");

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize(address initialOwner, address uts, address feeOracle, address l2Messenger) public initializer {
        __AccessControlDefaultAdminRules_init(3 days, initialOwner);

        require(feeOracle != address(0), "UTS: Invalid FeeOracle address");

        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        $.uts = IUniversalTimestamps(uts);
        $.feeOracle = IL1FeeOracle(feeOracle);
        // Start from 1 to use 0 as a sentinel value
        $.queueIndex = 1;
        $.confirmedIndex = 1;
        $.l2Messenger = IL2ScrollMessenger(l2Messenger);

        // Set up roles
        grantRole(FEE_COLLECTOR_ROLE, initialOwner);
        _setRoleAdmin(FEE_COLLECTOR_ROLE, DEFAULT_ADMIN_ROLE);
    }

    /// @inheritdoc IL2AnchoringManager
    function submitForL1Anchoring(bytes32 root, address refundAddress) external payable nonReentrant {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();

        require(address($.feeOracle) != address(0), "UTS: Oracle not set");

        uint256 requiredFee = $.feeOracle.getFloorFee();
        require(msg.value >= requiredFee, "UTS: Insufficient fee for L1 anchoring");

        // Call core contract to record the L2 timestamp.
        $.uts.attest(root);

        uint256 currentIndex = $.queueIndex++;
        $.items[currentIndex] = L2AnchoringManagerTypes.AnchoringItem({root: root, l1BlockNumber: 0});
        $.roots[root] = currentIndex;

        emit L1AnchoringQueued(root, currentIndex, requiredFee, block.number, block.timestamp);

        // refund fee to `refundAddress`
        unchecked {
            uint256 _refund = msg.value - requiredFee;
            if (_refund > 0) {
                (bool _success,) = refundAddress.call{value: _refund}("");
                require(_success, "Failed to refund the fee");
            }
        }
    }

    /// @inheritdoc IL2AnchoringManager
    function isConfirmed(bytes32 root) external view returns (bool) {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        uint256 index = $.roots[root];
        return index != 0 && index < $.confirmedIndex;
    }

    /// @inheritdoc IL2AnchoringManager
    function notifyAnchored(bytes32 expectedRoot, uint256 startIndex, uint256 count, uint256 l1BlockNumber) external {
        require(count > 0, "UTS: Count must be greater than zero");

        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        require(address($.l1Gateway) != address(0), "UTS: L1 Gateway not set");

        require(msg.sender == address($.l2Messenger), "UTS: Unauthorized caller");
        address l1Sender = $.l2Messenger.xDomainMessageSender();
        require(l1Sender == $.l1Gateway, "UTS: Invalid L1 sender");

        /// Require there's no pending batch to prevent overlapping batches which can cause state inconsistency
        require($.pendingBatch.count == 0, "UTS: Pending batch already exists");

        // Store the batch details for later finalization
        $.pendingBatch = L2AnchoringManagerTypes.L1Batch({
            expectedRoot: expectedRoot, startIndex: startIndex, count: count, l1BlockNumber: l1BlockNumber
        });

        emit L1BatchArrived(expectedRoot, startIndex, count, l1BlockNumber, block.number, block.timestamp);
    }

    /// @inheritdoc IL2AnchoringManager
    function finalizeBatch() external {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        L2AnchoringManagerTypes.L1Batch memory batch = $.pendingBatch;

        require(batch.count > 0, "UTS: No pending batch");

        bytes32[] memory leaves = new bytes32[](batch.count);
        for (uint256 i = 0; i < batch.count; i++) {
            uint256 index = batch.startIndex + i;
            L2AnchoringManagerTypes.AnchoringItem storage item = $.items[index];
            leaves[i] = item.root;
            item.l1BlockNumber = batch.l1BlockNumber;
        }

        bytes32 computedRoot = MerkleTree.computeRoot(leaves);
        require(computedRoot == batch.expectedRoot, "UTS: Invalid Merkle Root");

        $.confirmedIndex = batch.startIndex + batch.count;

        emit L1BatchFinalized(
            batch.expectedRoot, batch.startIndex, batch.count, batch.l1BlockNumber, block.number, block.timestamp
        );

        // Clear the pending batch
        delete $.pendingBatch;
    }

    // --- Admin Functions ---

    function setFeeOracle(address _oracle) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(address(_oracle) != address(0), "UTS: Invalid Oracle");
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        address oldOracle = address($.feeOracle);
        $.feeOracle = IL1FeeOracle(_oracle);
        emit FeeOracleUpdated(oldOracle, _oracle);
    }

    function setL1Gateway(address l1Gateway) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(l1Gateway != address(0), "UTS: Invalid L1 Gateway address");
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        address oldGateway = $.l1Gateway;
        $.l1Gateway = l1Gateway;
        emit L1GatewayUpdated(oldGateway, l1Gateway);
    }

    function setL2Messenger(address l2Messenger) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(l2Messenger != address(0), "UTS: Invalid L2 Messenger address");
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        IL2ScrollMessenger messenger = IL2ScrollMessenger(l2Messenger);
        // Sanity check to ensure it's a valid messenger
        require(
            messenger.xDomainMessageSender() == ScrollConstants.DEFAULT_XDOMAIN_MESSAGE_SENDER,
            "UTS: Invalid L2 Messenger"
        );
        $.l2Messenger = messenger;
        emit L2MessengerUpdated(address($.l2Messenger), l2Messenger);
    }

    /// @inheritdoc IL2AnchoringManager
    function clearBatch() external onlyRole(DEFAULT_ADMIN_ROLE) {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        delete $.pendingBatch;
    }

    /// @inheritdoc IL2AnchoringManager
    function withdrawFees(address to, uint256 amount) external nonReentrant onlyRole(FEE_COLLECTOR_ROLE) {
        require(to != address(0), "UTS: Invalid address");
        require(amount > 0 && amount <= address(this).balance, "UTS: Invalid amount");

        (bool success,) = payable(to).call{value: amount}("");
        require(success, "UTS: Withdrawal failed");

        emit FeesWithdrawn(to, amount);
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}

    receive() external payable {}
}
