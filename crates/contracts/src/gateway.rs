mod inner {
    alloy_sol_types::sol! {
        #[sol(rpc)]
        interface IL1AnchoringGateway {
            /// Emitted when a new batch of Merkle roots is submitted to L1 for anchoring.
            event BatchSubmitted(
                bytes32 indexed merkleRoot, uint256 indexed startIndex, uint256 count, address indexed submitter
            );

            /// Submit a SINGLE aggregated Merkle Root to L1 and trigger L2 verification.
            ///
            /// # Arguments
            /// - `merkleRoot` The root of the Merkle Tree containing all roots in this batch.
            /// - `startIndex` The queue index of the first root in this batch.
            /// - `count` The number of roots in this batch.
            /// - `gasLimit` The gas limit for L2 execution of this batch. Caller should estimate
            /// the gas cost based on the batch size and current L2 gas price, and provide enough
            /// ETH to cover both L1 Gas and L2 Execution Gas.
            /// - `msg.value` Caller must send enough ETH to cover L1 Gas + L2 Execution Gas.
            ///
            function submitBatch(bytes32 merkleRoot, uint256 startIndex, uint256 count, uint256 gasLimit) external payable;
        }
    }
    pub use IL1AnchoringGateway::*;
}

pub use inner::IL1AnchoringGatewayInstance as L1AnchoringGateway;

/// events
pub mod events {
    pub use super::inner::BatchSubmitted;
}

/// calls
pub mod calls {
    pub use super::inner::submitBatchCall as SubmitBatchCall;
}
