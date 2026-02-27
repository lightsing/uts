// SPDX-License-Identifier: MIT

pragma solidity ^0.8.29;

library L2AnchoringManagerTypes {
    /// @notice Attestation struct to hold timestamp and block number for each attested root
    struct AnchoringItem {
        bytes32 root;
        uint256 l1BlockNumber;
    }
}
