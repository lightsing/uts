// SPDX-License-Identifier: MIT

pragma solidity ^0.8.24;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {ReentrancyGuardTransient} from "@openzeppelin/contracts/utils/ReentrancyGuardTransient.sol";
import {IL1AnchoringManager} from "./IL1AnchoringManager.sol";
import {L1AnchoringManagerStorage} from "./L1AnchoringManagerStorage.sol";
import {IL1FeeOracle} from "../oracle/IL1FeeOracle.sol";
import {L1AnchoringManagerTypes} from "./L1AnchoringManagerTypes.sol";
import {MerkleTree} from "../../core/MerkleTree.sol";
import {Constants} from "../../Constants.sol";

contract L1AnchoringManager is
    Initializable,
    OwnableUpgradeable,
    UUPSUpgradeable,
    ReentrancyGuardTransient,
    IL1AnchoringManager
{
    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize(address initialOwner, IL1FeeOracle feeOracle) public initializer {
        __Ownable_init(initialOwner);

        require(address(feeOracle) != address(0), "UTS: Invalid FeeOracle address");

        L1AnchoringManagerStorage.Storage storage $ = L1AnchoringManagerStorage.get();
        $.feeOracle = feeOracle;
        $.feeCollector = initialOwner;
    }

    /// @inheritdoc IL1AnchoringManager
    function submitForL1Anchoring(bytes32 root) external payable nonReentrant {
        L1AnchoringManagerStorage.Storage storage $ = L1AnchoringManagerStorage.get();

        require(address($.feeOracle) != address(0), "UTS: Oracle not set");

        uint256 requiredFee = $.feeOracle.getFloorFee();
        require(msg.value >= requiredFee, "UTS: Insufficient fee for L1 anchoring");

        // Call core contract to record the L2 timestamp.
        Constants.UTS.attest(root);

        uint256 currentIndex = $.queueIndex++;
        $.items[currentIndex] = L1AnchoringManagerTypes.AnchoringItem({root: root, l1BlockNumber: 0});
        $.roots[root] = currentIndex;

        emit L1AnchoringQueued(root, currentIndex, msg.value, block.number, block.timestamp);
    }

    /// @inheritdoc IL1AnchoringManager
    function isConfirmed(bytes32 root) external view returns (bool) {
        L1AnchoringManagerStorage.Storage storage $ = L1AnchoringManagerStorage.get();
        uint256 index = $.roots[root];
        return index <= $.confirmedIndex;
    }

    /// @inheritdoc IL1AnchoringManager
    function confirmL1AnchoringBatch(bytes32 expectedRoot, uint256 startIndex, uint256 count, uint256 l1BlockNumber)
        external
    {
        // TODO: constraint on caller (e.g. only L1 Scroll Messenger or a designated relayer)
        L1AnchoringManagerStorage.Storage storage $ = L1AnchoringManagerStorage.get();

        bytes32[] memory leaves = new bytes32[](count);
        for (uint256 i = 0; i < count; i++) {
            uint256 index = startIndex + i;
            L1AnchoringManagerTypes.AnchoringItem storage item = $.items[index];
            leaves[i] = item.root;
            item.l1BlockNumber = l1BlockNumber;
        }

        bytes32 computedRoot = MerkleTree.computeRoot(leaves);
        require(computedRoot == expectedRoot, "UTS: Invalid Merkle Root");

        $.confirmedIndex = startIndex + count;
    }

    // --- Admin Functions ---

    function setFeeOracle(address _oracle) external onlyOwner {
        require(address(_oracle) != address(0), "UTS: Invalid Oracle");

        L1AnchoringManagerStorage.Storage storage $ = L1AnchoringManagerStorage.get();

        $.feeOracle = IL1FeeOracle(_oracle);
        emit FeeParametersUpdated(_oracle, $.feeCollector);
    }

    function setFeeCollector(address _collector) external onlyOwner {
        require(_collector != address(0), "UTS: Invalid Collector");
        L1AnchoringManagerStorage.Storage storage $ = L1AnchoringManagerStorage.get();
        $.feeCollector = _collector;
        emit FeeParametersUpdated(address($.feeOracle), _collector);
    }

    /// @inheritdoc IL1AnchoringManager
    function withdrawFees(address to, uint256 amount) external nonReentrant {
        L1AnchoringManagerStorage.Storage storage $ = L1AnchoringManagerStorage.get();

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
