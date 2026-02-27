// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

interface IL1FeeOracle {
    /**
     * @notice Emitted when fee parameters are updated.
     * @param gasPerAttestation Estimated gas consumed on L1 per attestation (in a batch)
     * @param discountRatio The discount ratio applied to the baseline fee (scaled by 1e18)
     */
    event ParametersUpdated(uint256 gasPerAttestation, uint256 discountRatio);

    /**
     * @notice Return the current L1 Base Fee used for calculation.
     */
    function getL1BaseFee() external view returns (uint256);

    /**
     * @notice Calculate the theoretical baseline fee WITHOUT aggregation discount.
     * @dev Useful for debugging or showing users how much they save.
     * @return feePerAttestation The fee if the user submitted independently to L1.
     */
    function getFeePerAttestation() external view returns (uint256);

    /**
     * @notice Calculate the final fee a user must pay for L1 anchoring.
     * @return fee The required fee in Wei, applying the discount ratio.
     */
    function getFloorFee() external view returns (uint256);

    /**
     * @notice Return the current gas consumed on L1 per attestation.
     */
    function gasPerAttestation() external view returns (uint256);

    /**
     * @notice Return the current discount ratio applied to the baseline fee.
     */
    function discountRatio() external view returns (uint256);
}
