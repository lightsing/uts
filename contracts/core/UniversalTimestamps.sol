// SPDX-License-Identifier: MIT

pragma solidity ^0.8.24;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {IUniversalTimestamps} from "./IUniversalTimestamps.sol";
import {UniversalTimestampsTypes} from "./UniversalTimestampsTypes.sol";
import {UniversalTimestampsStorage} from "./UniversalTimestampsStorage.sol";

/**
 * @title UniversalTimestamps
 */
contract UniversalTimestamps is Initializable, OwnableUpgradeable, UUPSUpgradeable, IUniversalTimestamps {
    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize(address initialOwner) public initializer {
        __Ownable_init(initialOwner);
    }

    function timestamp(bytes32 root) external view returns (uint256) {
        return UniversalTimestampsStorage.get().timestamps[root].timestamp;
    }

    function blockNumberOf(bytes32 root) external view returns (uint256) {
        return UniversalTimestampsStorage.get().timestamps[root].blockNumber;
    }

    function getAttestation(bytes32 root) external view returns (UniversalTimestampsTypes.Attestation memory) {
        return UniversalTimestampsStorage.get().timestamps[root];
    }

    /**
     * @notice Attest Merkle Root
     * @param root The Merkle Root to be attested
     */
    function attest(bytes32 root) external {
        require(root != bytes32(0), "UTS: Root cannot be zero");

        UniversalTimestampsStorage.Storage storage $ = UniversalTimestampsStorage.get();
        require($.timestamps[root].timestamp == 0, "UTS: Root already attested");
        $.timestamps[root] =
            UniversalTimestampsTypes.Attestation({timestamp: block.timestamp, blockNumber: block.number});
        emit Attested(root, msg.sender, block.timestamp, block.number);
    }

    /**
     * @dev Authorizes an upgrade to `newImplementation`.
     *
     * This function is restricted to the contract owner via the {onlyOwner} modifier,
     * ensuring that only the owner can authorize upgrades to the implementation.
     */
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
}
