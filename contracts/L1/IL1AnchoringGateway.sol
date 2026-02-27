// SPDX-License-Identifier: MIT

pragma solidity ^0.8.24;

interface IL1AnchoringGateway {
    /**
     * @notice Submit a SINGLE aggregated Merkle Root to L1 and trigger L2 verification.
     * @param merkleRoot The root of the Merkle Tree containing all roots in this batch.
     * @param startIndex The queue index of the first root in this batch.
     * @param count The number of roots in this batch.
     * @dev Caller must send enough ETH to cover L1 Gas + L2 Execution Gas (which is now higher due to loop).
     */
    function submitBatch(bytes32 merkleRoot, uint256 startIndex, uint256 count) external payable;
}
