// SPDX-License-Identifier: MIT

pragma solidity ^0.8.24;

library UniversalTimestampsTypes {
    /// @notice Attestation struct to hold timestamp and block number for each attested root
    struct Attestation {
        uint256 timestamp;
        uint256 blockNumber;
    }
}
