// SPDX-License-Identifier: MIT

pragma solidity ^0.8.29;

library L2AnchoringManagerTypes {
    /// @notice Attestation struct to hold timestamp and block number for each attested root
    struct AnchoringItem {
        bytes32 root;
        address submitter;
        uint256 l1BlockNumber;
    }

    /// @notice Struct to hold L1 notification details for batch confirmation
    struct L1Batch {
        bytes32 expectedRoot;
        uint256 startIndex;
        uint256 count;
        uint256 l1BlockNumber;
    }
}
