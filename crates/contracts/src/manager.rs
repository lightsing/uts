mod inner {
    alloy_sol_types::sol! {
        #![sol(all_derives)]

        #[sol(rpc)]
        interface IL2AnchoringManager {
            /// Emitted when a user pays to have their root anchored to L1.
            event L1AnchoringQueued(
                bytes32 indexed attestationId,
                bytes32 indexed root,
                uint256 queueIndex,
                uint256 fee,
                uint256 blockNumber,
                uint256 timestamp
            );

            /// Emitted when L1 notifies that a batch of roots has been anchored on L1.
            /// - `claimedRoot` The Merkle root claimed to be anchored on L1.
            /// - `startIndex` The starting index of the batch in the queue.
            /// - `count` The number of items in the batch.
            /// - `l1BlockAttested` The L1 block number at which the batch was anchored. It would
            /// be 0 if the root was timestamped before the batch submission.
            /// - `l1TimestampAttested` The timestamp at which the batch was anchored on L1.
            /// - `l2BlockNumber` The L2 block number at which the notification is received.
            /// - `l2TimestampReceived` The timestamp when the notification is received.
            ///
            event L1BatchArrived(
                bytes32 indexed claimedRoot,
                uint256 indexed startIndex,
                uint256 count,
                uint256 l1BlockAttested,
                uint256 l1TimestampAttested,
                uint256 l2BlockNumber,
                uint256 l2TimestampReceived
            );

            /// Emitted when a batch of roots is finalized after L1 confirmation.
            /// - `merkleRoot` The Merkle root of the batch.
            /// - `startIndex` The starting index of the batch in the queue.
            /// - `count` The number of items in the batch.
            /// - `l1BlockAttested` The L1 block number at which the batch was anchored. It would be
            /// 0 if the root was timestamped before the batch submission.
            /// - `l1TimestampAttested` The timestamp at which the batch was anchored on L1.
            /// - `l2BlockNumber` The L2 block number at which the batch is finalized.
            /// - `l2TimestampFinalized` The timestamp when the batch is finalized.-
            ///
            event L1BatchFinalized(
                bytes32 indexed merkleRoot,
                uint256 indexed startIndex,
                uint256 count,
                uint256 l1BlockAttested,
                uint256 l1TimestampAttested,
                uint256 l2BlockNumber,
                uint256 l2TimestampFinalized
            );

            /// Emitted when a user claims their NFT after batch confirmation.
            ///
            event NFTClaimed(address indexed submitter, uint256 indexed tokenId, bytes32 indexed root, uint256 timestamp);

            /// see also submitForL1Anchoring(bytes32 attestationId, address refundAddress).
            function submitForL1Anchoring(bytes32 attestationId) external payable;

            /// Finalize the batch confirmation after receiving the L1 notification. This will
            /// verify the Merkle root and update the confirmed index. This can be called by anyone
            /// after the notification is received to save the cost of L2 execution since the cross
            /// chain gas price is higher than L2 execution.
            ///
            function finalizeBatch() external;

            /// Withdraw accumulated fees to the collector.
            ///
            function withdrawFees(address to, uint256 amount) external;
        }
    }

    pub use IL2AnchoringManager::*;
}

pub use inner::IL2AnchoringManagerInstance as L2AnchoringManager;

/// events
pub mod events {
    pub use super::inner::{L1AnchoringQueued, L1BatchArrived, L1BatchFinalized, NFTClaimed};
}

/// calls
pub mod calls {
    pub use super::inner::finalizeBatchCall as FinalizeBatchCall;

    /// admin calls
    pub mod admins {
        pub use crate::manager::inner::withdrawFeesCall as WithdrawFeesCall;
    }
}
