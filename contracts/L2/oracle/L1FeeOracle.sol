// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import {IL1GasPriceOracle} from "scroll-contracts/L2/predeploys/IL1GasPriceOracle.sol";
import {IL1FeeOracle} from "./IL1FeeOracle.sol";
import {
    AccessControlDefaultAdminRules
} from "@openzeppelin/contracts/access/extensions/AccessControlDefaultAdminRules.sol";

/**
 * @title L1FeeOracle
 * @dev Calculates the fee required for L1 anchoring based on dynamic L1 gas prices
 *      and an aggregation discount ratio (<= 1.0).
 *
 * Formula: UserFee = (L1_Base_Fee * GasPerAttestation * DiscountRatio) / 1e18
 *
 * Semantic:
 * - _gasPerAttestation: The theoretical gas cost if a user submitted independently to L1 (e.g., 50,000).
 * - _discountRatio: The fraction (<= 1.0) representing the aggregated share + protocol margin (e.g., 0.005e18).
 */
contract L1FeeOracle is IL1FeeOracle, AccessControlDefaultAdminRules {
    // Predeploy contract on Scroll that provides the current L1 base fee.
    IL1GasPriceOracle public constant L1_GAS_PRICE_ORACLE =
        IL1GasPriceOracle(0x5300000000000000000000000000000000000002);

    bytes32 public constant UPDATER_ROLE = keccak256("UPDATER_ROLE");

    // Estimated gas consumed on L1 per attestation if submitted independently (without aggregation).
    uint256 public gasPerAttestation = 75_000;
    // Discount: The ratio (<= 1.0) representing the aggregated share + protocol margin.
    // Scaled by 1e18.
    // Example: 0.005e18 means user pays 0.5% of the baseline gas cost.
    // This value implicitly covers the actual batch share + server profit margin.
    uint256 public discountRatio = 1e18; // Default to no discount

    /**
     * @param initialOwner The owner of this oracle contract.
     */
    constructor(address initialOwner) AccessControlDefaultAdminRules(3 days, initialOwner) {
        _setRoleAdmin(UPDATER_ROLE, DEFAULT_ADMIN_ROLE);
        grantRole(UPDATER_ROLE, initialOwner);
    }

    /**
     * @notice Update parameters.
     * @param newGasPerAttestation Baseline gas for a single independent tx.
     * @param newDiscountRatio The aggregation ratio (must be <= 1e18).
     */
    function setParameters(uint256 newGasPerAttestation, uint256 newDiscountRatio) external onlyRole(UPDATER_ROLE) {
        require(newGasPerAttestation > 0, "L1FeeOracle: Gas must be positive");
        require(newDiscountRatio > 0 && newDiscountRatio <= 1e18, "L1FeeOracle: Ratio must be between 0 and 1.0");

        gasPerAttestation = newGasPerAttestation;
        discountRatio = newDiscountRatio;

        emit ParametersUpdated(newGasPerAttestation, newDiscountRatio);
    }

    /// @inheritdoc IL1FeeOracle
    function getL1BaseFee() external view returns (uint256) {
        return L1_GAS_PRICE_ORACLE.l1BaseFee();
    }

    /// @inheritdoc IL1FeeOracle
    function getFeePerAttestation() external view returns (uint256) {
        uint256 l1BaseFee = L1_GAS_PRICE_ORACLE.l1BaseFee();
        return l1BaseFee * gasPerAttestation;
    }

    /// @inheritdoc IL1FeeOracle
    function getFloorFee() external view returns (uint256) {
        uint256 l1BaseFee = L1_GAS_PRICE_ORACLE.l1BaseFee();
        // Calculate the fee with discount applied
        return (l1BaseFee * gasPerAttestation * discountRatio) / 1e18;
    }

    /// @inheritdoc IL1FeeOracle
    function getGasPerAttestation() external view returns (uint256) {
        return gasPerAttestation;
    }

    /// @inheritdoc IL1FeeOracle
    function getDiscountRatio() external view returns (uint256) {
        return discountRatio;
    }
}
