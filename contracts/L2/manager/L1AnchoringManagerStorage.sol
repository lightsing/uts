// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IL1FeeOracle} from "../oracle/IL1FeeOracle.sol";
import {L1AnchoringManagerTypes} from "./L1AnchoringManagerTypes.sol";

/**
 * @dev Library containing the ERC-7201 namespace constant.
 * This keeps the implementation detail hidden from the interface.
 */
library L1AnchoringManagerStorage {
    string internal constant NAMESPACE = "uts.storage.L1AnchoringManager";

    /// @dev keccak256(abi.encode(uint256(keccak256("uts.storage.L1AnchoringManager")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 internal constant SLOT = 0x9831cc7956aa6e272a6b3f7bd193bca727880ec1ca574ef61afc1d64fc9e5000;

    /// @custom:storage-location erc7201:uts.storage.L1AnchoringManager
    struct Storage {
        IL1FeeOracle feeOracle;
        address feeCollector;
        uint256 queueIndex;
        uint256 confirmedIndex;
        mapping(uint256 => L1AnchoringManagerTypes.AnchoringItem) items;
        mapping(bytes32 => uint256) roots; // Mapping to track submitted roots for quick lookup
    }

    function get() internal pure returns (L1AnchoringManagerStorage.Storage storage $) {
        assembly ("memory-safe") {
            $.slot := SLOT
        }
    }
}
