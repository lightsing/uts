// SPDX-License-Identifier: MIT

pragma solidity ^0.8.29;

interface IUniversalTimestamps {
    /// @notice Attestation struct to hold timestamp and block number for each attested root
    struct Attestation {
        uint256 timestamp;
        uint256 blockNumber;
    }

    /// @notice Emitted when a new Merkle root is attested with its timestamp and block number
    event Attested(bytes32 indexed root, address indexed attester, uint256 timestamp, uint256 blockNumber);

    /// @notice Returns the timestamp associated with a given Merkle root.
    /// @param root The Merkle root for which to retrieve the timestamp.
    /// @return The timestamp at which the root was attested, or 0 if the root has not been attested.
    /// @return A boolean indicating whether the root has been attested.
    function timestamp(bytes32 root) external view returns (uint256, bool);

    /// @notice Returns the block number associated with a given Merkle root.
    /// @param root The Merkle root for which to retrieve the block number.
    /// @return The block number at which the root was attested, or 0 if the root has not been attested.
    /// @return A boolean indicating whether the root has been attested.
    function blockNumberOf(bytes32 root) external view returns (uint256, bool);

    /// @notice Returns the full attestation (timestamp and block number) for a given Merkle root.
    /// @param root The Merkle root for which to retrieve the attestation.
    /// @return The attestation for the given root, or default values if the root has not been attested.
    /// @return A boolean indicating whether the root has been attested.
    function getAttestation(bytes32 root) external view returns (Attestation memory, bool);

    /// @notice Attests a new Merkle root with the current timestamp and block number.
    /// @param root The Merkle root to be attested.
    /// @return The attestation for the newly attested root.
    function attest(bytes32 root) external returns (Attestation memory);
}
