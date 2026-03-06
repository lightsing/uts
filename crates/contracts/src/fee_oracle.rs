mod inner {
    alloy_sol_types::sol! {
        #![sol(all_derives)]

        #[sol(rpc)]
        interface IFeeOracle {
            /// Calculate the final fee a user must pay for L1 anchoring.
            ///
            /// # Returns
            /// fee The required fee in Wei.
            ///
            function getFloorFee() external view returns (uint256);

            function setL1Overhead(uint256 l1Overhead) external;
            function setL1FeeScalar(uint256 l1FeeScalar) external;
            function setL1GasEstimated(uint256 l1GasEstimated) external;
            function setCrossDomainGasEstimated(uint256 crossDomainGasEstimated) external;
            function setL2ExecutionScalar(uint256 l2ExecutionScalar) external;
            function setL2ExecutionOverhead(uint256 l2ExecutionOverhead) external;
            function setExpectedBatchSize(uint256 expectedBatchSize) external;
            function setFeeMultiplier(uint256 feeMultiplier) external;
        }
    }
}

pub use inner::IFeeOracle::IFeeOracleInstance as FeeOracle;

/// calls
pub mod calls {
    pub use crate::fee_oracle::inner::IFeeOracle::getFloorFeeCall as GetFloorFeeCall;

    /// Admin calls
    pub mod admins {
        pub use crate::fee_oracle::inner::IFeeOracle::{
            setCrossDomainGasEstimatedCall as SetCrossDomainGasEstimatedCall,
            setExpectedBatchSizeCall as SetExpectedBatchSizeCall,
            setFeeMultiplierCall as SetFeeMultiplierCall, setL1FeeScalarCall as SetL1FeeScalarCall,
            setL1GasEstimatedCall as SetL1GasEstimatedCall, setL1OverheadCall as SetL1OverheadCall,
            setL2ExecutionOverheadCall as SetL2ExecutionOverheadCall,
            setL2ExecutionScalarCall as SetL2ExecutionScalarCall,
        };
    }
}
