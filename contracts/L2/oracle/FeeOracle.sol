// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

import {IL1GasPriceOracle} from "scroll-contracts/L2/predeploys/IL1GasPriceOracle.sol";
import {IFeeOracle} from "./IFeeOracle.sol";
import {
    AccessControlDefaultAdminRules
} from "@openzeppelin/contracts/access/extensions/AccessControlDefaultAdminRules.sol";

/**
 * @title FeeOracle
 */
contract FeeOracle is IFeeOracle, AccessControlDefaultAdminRules {
    // Predeploy contract on Scroll that provides the current L1 base fee.
    IL1GasPriceOracle public constant L1_GAS_PRICE_ORACLE =
        IL1GasPriceOracle(0x5300000000000000000000000000000000000002);

    bytes32 public constant UPDATER_ROLE = keccak256("UPDATER_ROLE");

    uint256 private constant PRECISION = 1e18;

    /// @notice value from SystemConfig contract on L1
    uint256 public l1Overhead = 39_200_000;

    /// @notice value from SystemConfig contract on L1
    uint256 public l1FeeScalar = 17e14;

    /// @notice gas required to attest a batch on L1
    uint256 public l1GasEstimated = 200_000;

    /// @notice gas required for L1->L2 message to notify L2 manager of new batch submission
    uint256 public crossDomainGasEstimated = 110_000;

    /// @notice scalar for L2 execution gas estimation for each additional batch item.
    uint256 public l2ExecutionScalar = 26_000;

    /// @notice overhead for L2 execution gas estimation
    uint256 public l2ExecutionOverhead = 0;

    /// @notice expected batch size for fee calculation
    uint256 public expectedBatchSize = 256;

    /// @notice multiplier to apply on top of the estimated cost to determine the floor fee.
    uint256 public feeMultiplier = 15e17;

    /**
     * @param initialOwner The owner of this oracle contract.
     */
    constructor(address initialOwner) AccessControlDefaultAdminRules(3 days, initialOwner) {
        _setRoleAdmin(UPDATER_ROLE, DEFAULT_ADMIN_ROLE);
        grantRole(UPDATER_ROLE, initialOwner);
    }

    /// @inheritdoc IFeeOracle
    function getFloorFee() external view returns (uint256) {
        uint256 estimatedCost = _estimateBatchCost();
        return (estimatedCost * feeMultiplier) / expectedBatchSize / PRECISION;
    }

    function _estimateBatchCost() internal view returns (uint256) {
        uint256 l1 = L1_GAS_PRICE_ORACLE.l1BaseFee() * l1GasEstimated;
        uint256 crossDomain = _getCrossDomainGasPrice() * crossDomainGasEstimated;
        uint256 l2 = block.basefee * _getL2ExecutionGas(expectedBatchSize);
        return l1 + crossDomain + l2;
    }

    function _getL2ExecutionGas(uint256 batchSize) internal view returns (uint256) {
        return l2ExecutionScalar * batchSize + l2ExecutionOverhead;
    }

    /// @notice formula from IL1MessageQueueV2's estimateL2BaseFee function
    function _getCrossDomainGasPrice() internal view returns (uint256) {
        uint256 l1BaseFee = L1_GAS_PRICE_ORACLE.l1BaseFee();
        return (l1BaseFee * l1FeeScalar) / PRECISION + l1Overhead;
    }

    // -- Admin functions to update fee parameters -- //

    function setL1Overhead(uint256 _l1Overhead) external onlyRole(UPDATER_ROLE) {
        l1Overhead = _l1Overhead;
        emit L1OverheadUpdated(_l1Overhead);
    }

    function setL1FeeScalar(uint256 _l1FeeScalar) external onlyRole(UPDATER_ROLE) {
        l1FeeScalar = _l1FeeScalar;
        emit L1FeeScalarUpdated(_l1FeeScalar);
    }

    function setL1GasEstimated(uint256 _l1GasEstimated) external onlyRole(UPDATER_ROLE) {
        l1GasEstimated = _l1GasEstimated;
        emit L1GasEstimatedUpdated(_l1GasEstimated);
    }

    function setCrossDomainGasEstimated(uint256 _crossDomainGasEstimated) external onlyRole(UPDATER_ROLE) {
        crossDomainGasEstimated = _crossDomainGasEstimated;
        emit CrossDomainGasEstimatedUpdated(_crossDomainGasEstimated);
    }

    function setL2ExecutionScalar(uint256 _l2ExecutionScalar) external onlyRole(UPDATER_ROLE) {
        l2ExecutionScalar = _l2ExecutionScalar;
        emit L2ExecutionScalarUpdated(_l2ExecutionScalar);
    }

    function setL2ExecutionOverhead(uint256 _l2ExecutionOverhead) external onlyRole(UPDATER_ROLE) {
        l2ExecutionOverhead = _l2ExecutionOverhead;
        emit L2ExecutionOverheadUpdated(_l2ExecutionOverhead);
    }

    function setExpectedBatchSize(uint256 _expectedBatchSize) external onlyRole(UPDATER_ROLE) {
        expectedBatchSize = _expectedBatchSize;
        emit ExpectedBatchSizeUpdated(_expectedBatchSize);
    }

    function setFeeMultiplier(uint256 _feeMultiplier) external onlyRole(UPDATER_ROLE) {
        feeMultiplier = _feeMultiplier;
        emit FeeMultiplierUpdated(_feeMultiplier);
    }
}
