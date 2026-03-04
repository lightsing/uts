// SPDX-License-Identifier: MIT

pragma solidity =0.8.28;

interface IL1AnchoringGateway {
    /// @notice Emitted when a new batch of Merkle roots is submitted to L1 for anchoring.
    event BatchSubmitted(
        bytes32 indexed merkleRoot, uint256 indexed startIndex, uint256 count, address indexed submitter
    );

    /// @notice Emitted when the L1 Scroll Messenger contract address is updated.
    event L1ScrollMessengerUpdated(address indexed oldMessenger, address indexed newMessenger);
    /// @notice Emitted when the L2 Anchoring Manager contract address is updated.
    event L2AnchoringManagerUpdated(address indexed oldManager, address indexed newManager);

    /**
     * @notice Submit a SINGLE aggregated Merkle Root to L1 and trigger L2 verification.
     * @param merkleRoot The root of the Merkle Tree containing all roots in this batch.
     * @param startIndex The queue index of the first root in this batch.
     * @param count The number of roots in this batch.
     * @param gasLimit The gas limit for L2 execution of this batch. Caller should estimate the gas cost based on the batch size and current L2 gas price, and provide enough ETH to cover both L1 Gas and L2 Execution Gas.
     * @dev Caller must send enough ETH to cover L1 Gas + L2 Execution Gas (which is now higher due to loop).
     */
    function submitBatch(bytes32 merkleRoot, uint256 startIndex, uint256 count, uint256 gasLimit) external payable;

    // -- Admin functions --
    function setL1ScrollMessenger(address newMessenger) external;
    function setL2AnchoringManager(address newManager) external;
}
