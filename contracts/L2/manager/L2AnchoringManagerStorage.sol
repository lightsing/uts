// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import {IL1FeeOracle} from "../oracle/IL1FeeOracle.sol";
import {L2AnchoringManagerTypes} from "./L2AnchoringManagerTypes.sol";
import {IUniversalTimestamps} from "../../core/IUniversalTimestamps.sol";
import {IL2ScrollMessenger} from "scroll-contracts/L2/IL2ScrollMessenger.sol";

/**
 * @dev Library containing the ERC-7201 namespace constant.
 * This keeps the implementation detail hidden from the interface.
 */
library L2AnchoringManagerStorage {
    string internal constant NAMESPACE = "uts.storage.L2AnchoringManager";

    /// @dev keccak256(abi.encode(uint256(keccak256("uts.storage.L2AnchoringManager")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 internal constant SLOT = 0x9831cc7956aa6e272a6b3f7bd193bca727880ec1ca574ef61afc1d64fc9e5000;

    /// @custom:storage-location erc7201:uts.storage.L2AnchoringManager
    struct Storage {
        IUniversalTimestamps uts;
        IL1FeeOracle feeOracle;
        /// @notice L1 contract that sends messages to this manager on L2
        address l1Messenger;
        /// @notice Executor for L1 -> L2 messages
        IL2ScrollMessenger l2Messenger;
        /// @notice L1 sender address
        address l1Gateway;
        /// @notice Address that collects the fees
        address feeCollector;
        /// @notice Queue index for the next anchoring item to be added
        uint256 queueIndex;
        /// @notice Next index of the anchoring item to be confirmed
        uint256 confirmedIndex;
        mapping(uint256 => L2AnchoringManagerTypes.AnchoringItem) items;
        mapping(bytes32 => uint256) roots; // Mapping to track submitted roots for quick lookup
    }

    function get() internal pure returns (L2AnchoringManagerStorage.Storage storage $) {
        assembly ("memory-safe") {
            $.slot := SLOT
        }
    }
}
