// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import {UniversalTimestampsTypes} from "./UniversalTimestampsTypes.sol";

/**
 * @dev Library containing the ERC-7201 namespace constant.
 * This keeps the implementation detail hidden from the interface.
 */
library UniversalTimestampsStorage {
    string internal constant NAMESPACE = "uts.storage.UniversalTimestamps";

    /// @dev keccak256(abi.encode(uint256(keccak256("uts.storage.UniversalTimestamps")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 internal constant SLOT = 0x500a69046951d8ea21a7dabf6fe6e1792e3ffa4dc61a276ae65b0e2b03468100;

    /// @custom:storage-location erc7201:uts.storage.UniversalTimestamps
    struct Storage {
        mapping(bytes32 => UniversalTimestampsTypes.Attestation) timestamps;
    }

    function get() internal pure returns (UniversalTimestampsStorage.Storage storage $) {
        assembly ("memory-safe") {
            $.slot := SLOT
        }
    }
}
