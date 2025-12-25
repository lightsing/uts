// SPDX-License-Identifier: MIT

pragma solidity ^0.8.24;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {IUniversalTimestamps} from "./IUniversalTimestamps.sol";
import {SlotDerivation} from "@openzeppelin/contracts/utils/SlotDerivation.sol";

/**
 * @title UniversalTimestamps
 * @dev
 */
contract UniversalTimestamps is Initializable, OwnableUpgradeable, UUPSUpgradeable, IUniversalTimestamps {
    using SlotDerivation for string;

    string private constant _NAMESPACE = "uts.storage.UniversalTimestamps";

    /// @custom:storage-location erc7201:uts.storage.UniversalTimestamps
    struct UniversalTimestampsStorage {
        mapping(bytes32 => uint256) timestamps;
    }

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize(address initialOwner) public initializer {
        __Ownable_init(initialOwner);
    }

    function _getUniversalTimestampsStorage() private pure returns (UniversalTimestampsStorage storage $) {
        bytes32 slot = _NAMESPACE.erc7201Slot();
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    function timestamp(bytes32 root) external view returns (uint256) {
        return _getUniversalTimestampsStorage().timestamps[root];
    }

    /**
     * @notice Attest Merkle Root
     * @param root The Merkle Root to be attested
     */
    function attest(bytes32 root) external {
        require(root != bytes32(0), "UTS: Root cannot be zero");

        UniversalTimestampsStorage storage $ = _getUniversalTimestampsStorage();
        if ($.timestamps[root] == 0) {
            $.timestamps[root] = block.timestamp;
            emit Attested(root, msg.sender, block.timestamp);
        }
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
}
