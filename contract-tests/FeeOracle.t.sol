// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

import {Test} from "forge-std/Test.sol";
import {FeeOracle} from "../contracts/L2/oracle/FeeOracle.sol";
import {IFeeOracle} from "../contracts/L2/oracle/IFeeOracle.sol";

contract FeeOracleTest is Test {
    FeeOracle oracle;
    address owner = address(this);
    address updater = address(0xA1);

    address constant L1_GAS_PRICE_ORACLE = 0x5300000000000000000000000000000000000002;

    function setUp() public {
        // Mock the L1 gas price oracle predeploy
        vm.mockCall(L1_GAS_PRICE_ORACLE, abi.encodeWithSignature("l1BaseFee()"), abi.encode(uint256(50 gwei)));

        oracle = new FeeOracle(owner);
        oracle.grantRole(oracle.UPDATER_ROLE(), updater);
    }

    function test_GetFloorFee() public view {
        uint256 fee = oracle.getFloorFee();
        assertTrue(fee > 0, "Floor fee should be greater than zero");

        // Manually compute expected fee:
        // l1 = 50 gwei * 350_000 = 17_500_000 gwei
        // crossDomainGasPrice = (50 gwei * 17e14) / 1e18 + 39_200_000 = 85 + 39_200_000 = 39_200_085
        // crossDomain = 39_200_085 * 110_000 = 4_312_009_350_000
        // l2ExecutionGas = 3500 * 256 + 35_000 = 931_000
        // l2 = block.basefee * 931_000
        // total = l1 + crossDomain + l2
        // fee = total * 15e17 / 256 / 1e18

        uint256 l1 = 50 gwei * 350_000;
        uint256 crossDomainGasPrice = (50 gwei * 17e14) / 1e18 + 39_200_000;
        uint256 crossDomain = crossDomainGasPrice * 110_000;
        uint256 l2ExecutionGas = 3500 * 256 + 35_000;
        uint256 l2 = block.basefee * l2ExecutionGas;
        uint256 total = l1 + crossDomain + l2;
        uint256 expectedFee = (total * 15e17) / 256 / 1e18;

        assertEq(fee, expectedFee, "Fee calculation mismatch");
    }

    function test_SetParameters_OnlyUpdater() public {
        // Non-updater should fail
        vm.prank(address(0xBAD));
        vm.expectRevert();
        oracle.setL1Overhead(100);

        // Updater should succeed
        vm.prank(updater);
        oracle.setL1Overhead(100);
        assertEq(oracle.l1Overhead(), 100);
    }

    function test_SetL1Overhead() public {
        vm.prank(updater);
        vm.expectEmit(false, false, false, true);
        emit IFeeOracle.L1OverheadUpdated(12345);
        oracle.setL1Overhead(12345);
        assertEq(oracle.l1Overhead(), 12345);
    }

    function test_SetL1FeeScalar() public {
        vm.prank(updater);
        vm.expectEmit(false, false, false, true);
        emit IFeeOracle.L1FeeScalarUpdated(99e14);
        oracle.setL1FeeScalar(99e14);
        assertEq(oracle.l1FeeScalar(), 99e14);
    }

    function test_SetExpectedBatchSize() public {
        vm.prank(updater);
        vm.expectEmit(false, false, false, true);
        emit IFeeOracle.ExpectedBatchSizeUpdated(512);
        oracle.setExpectedBatchSize(512);
        assertEq(oracle.expectedBatchSize(), 512);
    }

    function test_SetFeeMultiplier() public {
        vm.prank(updater);
        vm.expectEmit(false, false, false, true);
        emit IFeeOracle.FeeMultiplierUpdated(2e18);
        oracle.setFeeMultiplier(2e18);
        assertEq(oracle.feeMultiplier(), 2e18);
    }
}
