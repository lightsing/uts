// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

interface IFeeOracle {
    /// @notice Emitted when the L1 fee parameters are updated.
    event L1OverheadUpdated(uint256 l1Overhead);
    /// @notice Emitted when the L1 fee scalar is updated.
    event L1FeeScalarUpdated(uint256 l1FeeScalar);
    /// @notice Emitted when the L1 gas estimation is updated.
    event L1GasEstimatedUpdated(uint256 l1GasEstimated);
    /// @notice Emitted when the cross-domain gas estimation is updated.
    event CrossDomainGasEstimatedUpdated(uint256 crossDomainGasEstimated);
    /// @notice Emitted when the L2 execution scalar is updated.
    event L2ExecutionScalarUpdated(uint256 l2ExecutionScalar);
    /// @notice Emitted when the L2 execution overhead is updated.
    event L2ExecutionOverheadUpdated(uint256 l2ExecutionOverhead);
    /// @notice Emitted when the expected batch size is updated.
    event ExpectedBatchSizeUpdated(uint256 expectedBatchSize);
    /// @notice Emitted when the fee multiplier is updated.
    event FeeMultiplierUpdated(uint256 feeMultiplier);

    /**
     * @notice Calculate the final fee a user must pay for L1 anchoring.
     * @return fee The required fee in Wei, applying the discount ratio.
     */
    function getFloorFee() external view returns (uint256);

    // -- Admin functions to update fee parameters --
    function setL1Overhead(uint256 l1Overhead) external;
    function setL1FeeScalar(uint256 l1FeeScalar) external;
    function setL1GasEstimated(uint256 l1GasEstimated) external;
    function setCrossDomainGasEstimated(uint256 crossDomainGasEstimated) external;
    function setL2ExecutionScalar(uint256 l2ExecutionScalar) external;
    function setL2ExecutionOverhead(uint256 l2ExecutionOverhead) external;
    function setExpectedBatchSize(uint256 expectedBatchSize) external;
    function setFeeMultiplier(uint256 feeMultiplier) external;
}
