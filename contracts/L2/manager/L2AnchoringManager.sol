// SPDX-License-Identifier: MIT

pragma solidity ^0.8.29;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {ReentrancyGuardTransient} from "@openzeppelin/contracts/utils/ReentrancyGuardTransient.sol";
import {IL2AnchoringManager} from "./IL2AnchoringManager.sol";
import {L2AnchoringManagerStorage} from "./L2AnchoringManagerStorage.sol";
import {IL1FeeOracle} from "../oracle/IL1FeeOracle.sol";
import {L2AnchoringManagerTypes} from "./L2AnchoringManagerTypes.sol";
import {MerkleTree} from "../../core/MerkleTree.sol";
import {IUniversalTimestamps} from "../../core/IUniversalTimestamps.sol";
import {IL2ScrollMessenger} from "scroll-contracts/L2/IL2ScrollMessenger.sol";
import {AddressAliasHelper} from "scroll-contracts/libraries/common/AddressAliasHelper.sol";
import {ScrollConstants} from "scroll-contracts/libraries/constants/ScrollConstants.sol";

contract L2AnchoringManager is
    Initializable,
    OwnableUpgradeable,
    UUPSUpgradeable,
    ReentrancyGuardTransient,
    IL2AnchoringManager
{
    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize(address initialOwner, address uts, address feeOracle, address l1Messenger, address l2Messenger)
        public
        initializer
    {
        __Ownable_init(initialOwner);

        require(feeOracle != address(0), "UTS: Invalid FeeOracle address");

        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        $.uts = IUniversalTimestamps(uts);
        $.feeOracle = IL1FeeOracle(feeOracle);
        $.l1Messenger = l1Messenger;
        $.l2Messenger = IL2ScrollMessenger(l2Messenger);
        $.feeCollector = initialOwner;
    }

    /// @inheritdoc IL2AnchoringManager
    function submitForL1Anchoring(bytes32 root) external payable nonReentrant {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();

        require(address($.feeOracle) != address(0), "UTS: Oracle not set");

        uint256 requiredFee = $.feeOracle.getFloorFee();
        require(msg.value >= requiredFee, "UTS: Insufficient fee for L1 anchoring");

        // Call core contract to record the L2 timestamp.
        $.uts.attest(root);

        uint256 currentIndex = $.queueIndex++;
        $.items[currentIndex] = L2AnchoringManagerTypes.AnchoringItem({root: root, l1BlockNumber: 0});
        $.roots[root] = currentIndex;

        emit L1AnchoringQueued(root, currentIndex, msg.value, block.number, block.timestamp);
    }

    /// @inheritdoc IL2AnchoringManager
    function isConfirmed(bytes32 root) external view returns (bool) {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        uint256 index = $.roots[root];
        return index < $.confirmedIndex;
    }

    /// @inheritdoc IL2AnchoringManager
    function confirmL1AnchoringBatch(bytes32 expectedRoot, uint256 startIndex, uint256 count, uint256 l1BlockNumber)
        external
    {
        require(count > 0, "UTS: Count must be greater than zero");

        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        require(address($.l1Gateway) != address(0), "UTS: L1 Gateway not set");

        require(msg.sender == address($.l2Messenger), "UTS: Unauthorized caller");
        address l1Sender = $.l2Messenger.xDomainMessageSender();
        require(l1Sender == $.l1Gateway, "UTS: Invalid L1 sender");

        bytes32[] memory leaves = new bytes32[](count);
        for (uint256 i = 0; i < count; i++) {
            uint256 index = startIndex + i;
            L2AnchoringManagerTypes.AnchoringItem storage item = $.items[index];
            leaves[i] = item.root;
            item.l1BlockNumber = l1BlockNumber;
        }

        bytes32 computedRoot = MerkleTree.computeRoot(leaves);
        require(computedRoot == expectedRoot, "UTS: Invalid Merkle Root");

        emit L1AnchoringBatchConfirmed(computedRoot, startIndex, count, l1BlockNumber, block.number, block.timestamp);

        $.confirmedIndex = startIndex + count;
    }

    // --- Admin Functions ---

    function setFeeOracle(address _oracle) external onlyOwner {
        require(address(_oracle) != address(0), "UTS: Invalid Oracle");
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        $.feeOracle = IL1FeeOracle(_oracle);
        emit FeeParametersUpdated(_oracle, $.feeCollector);
    }

    function setFeeCollector(address collector) external onlyOwner {
        require(collector != address(0), "UTS: Invalid Collector");
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        $.feeCollector = collector;
        emit FeeParametersUpdated(address($.feeOracle), collector);
    }

    function setL1Gateway(address l1Gateway) external onlyOwner {
        require(l1Gateway != address(0), "UTS: Invalid L1 Gateway address");
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        $.l1Gateway = l1Gateway;
        emit L1GatewayUpdated(l1Gateway);
    }

    function setL1Messenger(address l1Messenger) external onlyOwner {
        require(l1Messenger != address(0), "UTS: Invalid L1 Messenger address");
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        $.l1Messenger = l1Messenger;
        emit L1MessengerUpdated(l1Messenger);
    }

    function setL2Messenger(address l2Messenger) external onlyOwner {
        require(l2Messenger != address(0), "UTS: Invalid L2 Messenger address");
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        IL2ScrollMessenger messenger = IL2ScrollMessenger(l2Messenger);
        // Sanity check to ensure it's a valid messenger
        require(
            messenger.xDomainMessageSender() == ScrollConstants.DEFAULT_XDOMAIN_MESSAGE_SENDER,
            "UTS: Invalid L2 Messenger"
        );
        $.l2Messenger = messenger;
        emit L2MessengerUpdated(l2Messenger);
    }

    /// @inheritdoc IL2AnchoringManager
    function withdrawFees(address to, uint256 amount) external nonReentrant {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();

        // Security: Allow either Owner or the designated Collector to withdraw
        require(msg.sender == owner() || msg.sender == $.feeCollector, "UTS: Unauthorized");

        require(to != address(0), "UTS: Invalid address");
        require(amount > 0 && amount <= address(this).balance, "UTS: Invalid amount");

        (bool success,) = payable(to).call{value: amount}("");
        require(success, "UTS: Withdrawal failed");

        emit FeesWithdrawn(to, amount);
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    receive() external payable {}
}
