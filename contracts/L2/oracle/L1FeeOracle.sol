// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {IL1GasPriceOracle} from "scroll-contracts/L2/predeploys/IL1GasPriceOracle.sol";
import {IL1FeeOracle} from "./IL1FeeOracle.sol";
import {Constants} from "../../Constants.sol";

/**
 * @title L1FeeOracle
 * @dev Calculates the fee required for L1 anchoring based on dynamic L1 gas prices
 *      and an aggregation discount ratio (< 1.0).
 *
 * Formula: UserFee = (L1_Base_Fee * GasPerAttestation * DiscountRatio) / 1e18
 *
 * Semantic:
 * - _gasPerAttestation: The theoretical gas cost if a user submitted independently to L1 (e.g., 50,000).
 * - _discountRatio: The fraction (< 1.0) representing the aggregated share + protocol margin (e.g., 0.005e18).
 */
contract L1FeeOracle is IL1FeeOracle, Ownable {
    // Estimated gas consumed on L1 per attestation (in a batch)
    uint256 private _gasPerAttestation;
    // Discount: The ratio (< 1.0) representing the aggregated share + protocol margin.
    // Scaled by 1e18.
    // Example: 0.005e18 means user pays 0.5% of the baseline gas cost.
    // This value implicitly covers the actual batch share + server profit margin.
    uint256 private _discountRatio;

    /**
     * @param initialOwner The owner of this oracle contract.
     * @param initialGasPerAttestation Initial estimate of gas units consumed on L1 per attestation.
     * @param initialDiscountRatio Initial discount ratio (scaled by 1e18) to apply to the baseline fee calculation.
     */
    constructor(address initialOwner, uint256 initialGasPerAttestation, uint256 initialDiscountRatio)
        Ownable(initialOwner)
    {
        require(initialDiscountRatio > 0, "L1FeeOracle: Ratio must be positive");

        _gasPerAttestation = initialGasPerAttestation;
        _discountRatio = initialDiscountRatio;
    }

    /**
     * @notice Update parameters.
     * @param newGasPerAttestation Baseline gas for a single independent tx.
     * @param newDiscountRatio The aggregation ratio (must be <= 1e18).
     */
    function setParameters(uint256 newGasPerAttestation, uint256 newDiscountRatio) external onlyOwner {
        require(newGasPerAttestation > 0, "L1FeeOracle: Gas must be positive");
        require(newDiscountRatio > 0 && newDiscountRatio <= 1e18, "L1FeeOracle: Ratio must be between 0 and 1.0");

        _gasPerAttestation = newGasPerAttestation;
        _discountRatio = newDiscountRatio;

        emit ParametersUpdated(newGasPerAttestation, newDiscountRatio);
    }

    /// @inheritdoc IL1FeeOracle
    function getL1BaseFee() external view returns (uint256) {
        return Constants.L1_GAS_PRICE_ORACLE.l1BaseFee();
    }

    /// @inheritdoc IL1FeeOracle
    function getFeePerAttestation() external view returns (uint256) {
        uint256 l1BaseFee = Constants.L1_GAS_PRICE_ORACLE.l1BaseFee();
        return (l1BaseFee * _gasPerAttestation) / 1e18;
    }

    /// @inheritdoc IL1FeeOracle
    function getFloorFee() external view returns (uint256) {
        uint256 l1BaseFee = Constants.L1_GAS_PRICE_ORACLE.l1BaseFee();
        // Calculate the fee with discount applied
        return (l1BaseFee * _gasPerAttestation * _discountRatio) / 1e18;
    }

    /// @inheritdoc IL1FeeOracle
    function gasPerAttestation() external view returns (uint256) {
        return _gasPerAttestation;
    }

    /// @inheritdoc IL1FeeOracle
    function discountRatio() external view returns (uint256) {
        return _discountRatio;
    }
}
