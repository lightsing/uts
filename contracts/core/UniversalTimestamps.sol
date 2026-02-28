// SPDX-License-Identifier: MIT

pragma solidity ^0.8.29;

import {IUniversalTimestamps} from "./IUniversalTimestamps.sol";

/**
 * @title UniversalTimestamps
 */
contract UniversalTimestamps is IUniversalTimestamps {
    mapping(bytes32 => Attestation) timestamps;

    function timestamp(bytes32 root) external view returns (uint256, bool) {
        Attestation memory attestation = timestamps[root];
        return (attestation.timestamp, attestation.timestamp != 0);
    }

    function blockNumberOf(bytes32 root) external view returns (uint256, bool) {
        Attestation memory attestation = timestamps[root];
        return (attestation.blockNumber, attestation.timestamp != 0);
    }

    function getAttestation(bytes32 root) external view returns (Attestation memory, bool) {
        Attestation memory attestation = timestamps[root];
        return (attestation, attestation.timestamp != 0);
    }

    /**
     * @notice Attest Merkle Root
     * @param root The Merkle Root to be attested
     * @return The attestation for the newly attested root.
     */
    function attest(bytes32 root) external returns (Attestation memory) {
        require(root != bytes32(0), "UTS: Root cannot be zero");
        require(timestamps[root].timestamp == 0, "UTS: Root already attested");

        timestamps[root] = Attestation({timestamp: block.timestamp, blockNumber: block.number});
        emit Attested(root, msg.sender, block.timestamp, block.number);
        return timestamps[root];
    }
}
