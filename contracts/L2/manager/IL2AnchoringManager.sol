// SPDX-License-Identifier: MIT

pragma solidity ^0.8.29;

interface IL2AnchoringManager {
    /// @notice Emitted when a user pays to have their root anchored to L1.
    event L1AnchoringQueued(
        bytes32 indexed root, uint256 queueIndex, uint256 fee, uint256 blockNumber, uint256 timestamp
    );

    /// @notice Emitted when a batch of roots is confirmed as anchored on L1.
    event L1AnchoringBatchConfirmed(
        bytes32 indexed aggregateRoot,
        uint256 indexed startIndex,
        uint256 count,
        uint256 l1BlockNumber,
        uint256 l2BlockNumber,
        uint256 timestamp
    );

    /// @notice Emitted when fee parameters are updated.
    event FeeParametersUpdated(address indexed feeOracle, address indexed feeCollector);
    /// @notice Emitted when fees are withdrawn by the fee collector.
    event FeesWithdrawn(address indexed to, uint256 amount);
    /// @notice Emitted when the L1 Gateway address is updated.
    event L1GatewayUpdated(address indexed l1Gateway);
    /// @notice Emitted when the L1 Messenger address is updated.
    event L1MessengerUpdated(address indexed l1Messenger);
    /// @notice Emitted when the L2 Messenger address is updated.
    event L2MessengerUpdated(address indexed l2Messenger);

    /**
     * @notice Submit a root for L2 timestamping + L1 anchoring.
     * @dev Requires msg.value >= Oracle calculated fee.
     */
    function submitForL1Anchoring(bytes32 root) external payable;

    /**
     * @notice Check if a root has been confirmed as anchored on L1.
     * @param root The Merkle root to check for confirmation.
     * @return True if the root has been confirmed as anchored on L1, false otherwise.
     */
    function isConfirmed(bytes32 root) external view returns (bool);

    /**
     * @notice Called by the L1AnchoringGateway to confirm a batch of anchored roots.
     * @param expectedRoot The expected Merkle root of the batch being confirmed.
     * @param startIndex The starting index of the batch in the queue.
     * @param count The number of items in the batch.
     * @param l1BlockNumber The L1 block number at which the batch was anchored.
     */
    function confirmL1AnchoringBatch(bytes32 expectedRoot, uint256 startIndex, uint256 count, uint256 l1BlockNumber)
        external;

    // --- Admin Functions ---

    function setFeeOracle(address oracle) external;
    function setFeeCollector(address collector) external;
    function setL1Gateway(address l1Gateway) external;
    function setL1Messenger(address l1Messenger) external;
    function setL2Messenger(address l2Messenger) external;

    /**
     * @notice Withdraw accumulated fees to the collector.
     * @dev Only callable by the feeCollector or Owner.
     */
    function withdrawFees(address to, uint256 amount) external;
}
