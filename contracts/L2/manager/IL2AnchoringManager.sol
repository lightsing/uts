// SPDX-License-Identifier: MIT

pragma solidity =0.8.28;

interface IL2AnchoringManager {
    /// @notice Emitted when a user pays to have their root anchored to L1.
    event L1AnchoringQueued(
        bytes32 indexed attestationId,
        bytes32 indexed root,
        uint256 queueIndex,
        uint256 fee,
        uint256 blockNumber,
        uint256 timestamp
    );

    /**
     * Emitted when L1 notifies that a batch of roots has been anchored on L1.
     * @param claimedRoot The Merkle root claimed to be anchored on L1.
     * @param startIndex The starting index of the batch in the queue.
     * @param count The number of items in the batch.
     * @param l1BlockAttested The L1 block number at which the batch was anchored. It would be 0 if the root was timestamped before the batch submission.
     * @param l1TimestampAttested The timestamp at which the batch was anchored on L1.
     * @param l2BlockNumber The L2 block number at which the notification is received.
     * @param l2TimestampReceived The timestamp when the notification is received.
     */
    event L1BatchArrived(
        bytes32 indexed claimedRoot,
        uint256 indexed startIndex,
        uint256 count,
        uint256 l1BlockAttested,
        uint256 l1TimestampAttested,
        uint256 l2BlockNumber,
        uint256 l2TimestampReceived
    );

    /**
     * Emitted when a batch of roots is finalized after L1 confirmation.
     * @param merkleRoot The Merkle root of the batch.
     * @param startIndex The starting index of the batch in the queue.
     * @param count The number of items in the batch.
     * @param l1BlockAttested The L1 block number at which the batch was anchored. It would be 0 if the root was timestamped before the batch submission.
     * @param l1TimestampAttested The timestamp at which the batch was anchored on L1.
     * @param l2BlockNumber The L2 block number at which the batch is finalized.
     * @param l2TimestampFinalized The timestamp when the batch is finalized.
     */
    event L1BatchFinalized(
        bytes32 indexed merkleRoot,
        uint256 indexed startIndex,
        uint256 count,
        uint256 l1BlockAttested,
        uint256 l1TimestampAttested,
        uint256 l2BlockNumber,
        uint256 l2TimestampFinalized
    );

    /// @notice Emitted when a user claims their NFT after batch confirmation.
    event NFTClaimed(address indexed submitter, uint256 indexed tokenId, bytes32 indexed root, uint256 timestamp);

    /// @notice Emitted when a batch is cleared by the admin.
    event UtsUpdated(address indexed oldUts, address indexed newUts);
    /// @notice Emitted when fee parameters are updated.
    event FeeOracleUpdated(address indexed oldOracle, address indexed newOracle);
    /// @notice Emitted when fees are withdrawn by the fee collector.
    event FeesWithdrawn(address indexed to, uint256 amount);
    /// @notice Emitted when the L1 Gateway address is updated.
    event L1GatewayUpdated(address indexed oldGateway, address indexed newGateway);
    /// @notice Emitted when the L2 Messenger address is updated.
    event L2MessengerUpdated(address indexed oldMessenger, address indexed newMessenger);
    /// @notice Emitted when the base URI for token metadata is updated.
    event BaseURIUpdated(string oldBaseURI, string newBaseURI);

    /// @notice see also submitForL1Anchoring(bytes32 root, address refundAddress).
    function submitForL1Anchoring(bytes32 root) external payable;

    /**
     * @notice Submit a root for L2 timestamping + L1 anchoring.
     * @param root The Merkle root to be anchored on L1.
     * @param refundAddress The address to refund any excess fee after covering the required fee for L1 anchoring.
     * This allows users to get a refund if they overpay.
     * @dev Requires msg.value >= Oracle calculated fee.
     */
    function submitForL1Anchoring(bytes32 root, address refundAddress) external payable;

    /**
     * @notice Finalize the batch confirmation after receiving the L1 notification. This will verify the Merkle root
     * and update the confirmed index. This can be called by anyone after the notification is received to save the
     * cost of L2 execution since the cross chain gas price is higher than L2 execution.
     */
    function finalizeBatch() external;

    /**
     * @notice Check if a root has been confirmed as anchored on L1.
     * @param root The Merkle root to check for confirmation.
     * @return True if the root has been confirmed as anchored on L1, false otherwise.
     */
    function isConfirmed(bytes32 root) external view returns (bool);

    /// @notice Claim the NFT for a confirmed attestation.
    // forge-lint: disable-next-line(mixed-case-function)
    function claimNFT(bytes32 attestationId) external;

    /// @notice Returns the current base URI for token metadata
    function getBaseURI() external view returns (string memory);

    /**
     * @notice Called by the L1AnchoringGateway to notify that a batch of roots has been anchored on L1.
     * The cross chain gas price is higher than l2 execution, so we separate the notification and finalization into
     * two steps to save cost. The batch details will be stored when notifyAnchored is called, and the actual
     * confirmation will be done in finalizeBatch which can be called by anyone after the notification.
     *
     * @param expectedRoot The expected Merkle root of the batch being confirmed.
     * @param startIndex The starting index of the batch in the queue.
     * @param count The number of items in the batch.
     * @param l1Timestamp The timestamp at which the batch was anchored on L1.
     * @param l1BlockNumber The L1 block number at which the batch was anchored. It would be 0 if the root was timestamped before the batch submission.
     */
    function notifyAnchored(
        bytes32 expectedRoot,
        uint256 startIndex,
        uint256 count,
        uint256 l1Timestamp,
        uint256 l1BlockNumber
    ) external;

    // --- Admin Functions ---

    function setFeeOracle(address oracle) external;
    function setL1Gateway(address l1Gateway) external;
    function setL2Messenger(address l2Messenger) external;

    /**
     * @notice Clear the pending batch. This can be used by the admin to reset the state in case of an emergency or if the batch finalization fails due to unforeseen issues. It allows the contract to accept new batches without being stuck on a pending batch.
     */
    function clearBatch() external;

    /**
     * @notice Withdraw accumulated fees to the collector.
     * @dev Only callable by the feeCollector or Owner.
     */
    function withdrawFees(address to, uint256 amount) external;
}
