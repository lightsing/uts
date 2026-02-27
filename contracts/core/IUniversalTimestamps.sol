// SPDX-License-Identifier: MIT

pragma solidity ^0.8.24;

import {UniversalTimestampsTypes} from "./UniversalTimestampsTypes.sol";

interface IUniversalTimestamps {
    /// @notice Emitted when a new Merkle root is attested with its timestamp and block number
    event Attested(bytes32 indexed root, address indexed attester, uint256 timestamp, uint256 blockNumber);

    /// @notice Returns the timestamp associated with a given Merkle root. Reuturns 0 if the root has not been attested.
    function timestamp(bytes32 root) external view returns (uint256);

    /// @notice Returns the block number associated with a given Merkle root. Returns 0 if the root has not been attested.
    function blockNumberOf(bytes32 root) external view returns (uint256);

    /// @notice Returns the full attestation (timestamp and block number) for a given Merkle root. Returns default values if the root has not been attested.
    function getAttestation(bytes32 root) external view returns (UniversalTimestampsTypes.Attestation memory);

    /// @notice Attests a new Merkle root with the current timestamp and block number.
    function attest(bytes32 root) external;
}
