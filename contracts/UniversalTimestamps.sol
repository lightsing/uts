// SPDX-License-Identifier: MIT

pragma solidity ^0.8.24;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {IUniversalTimestamps} from "./IUniversalTimestamps.sol";
import {SlotDerivation} from "@openzeppelin/contracts/utils/SlotDerivation.sol";

/**
 * @title UniversalTimestamps
 * @dev Records and exposes timestamps for attested Merkle roots using ERC-7201
 * namespaced storage (`uts.storage.UniversalTimestamps`) derived via
 * {SlotDerivation}, and is implemented as a UUPS upgradeable contract via
 * OpenZeppelin's Initializable, OwnableUpgradeable, and UUPSUpgradeable
 * base contracts. Storage is kept in a dedicated namespaced struct to remain
 * layout-compatible across upgrades, while upgrades are authorized by the
 * contract owner through {_authorizeUpgrade}.
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

    /**
     * @dev Authorizes an upgrade to `newImplementation`.
     *
     * This function is restricted to the contract owner via the {onlyOwner} modifier,
     * ensuring that only the owner can authorize upgrades to the implementation.
     */
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
}
