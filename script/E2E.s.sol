// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Script, console} from "forge-std/Script.sol";
import {IL1AnchoringManager} from "../contracts/L2/manager/IL1AnchoringManager.sol";
import {IL1AnchoringGateway} from "../contracts/L1/IL1AnchoringGateway.sol";
import {IL1FeeOracle} from "../contracts/L2/oracle/IL1FeeOracle.sol";

contract SubmitAnchoring is Script {
    function run() public {
        address l1AnchoringManager = vm.envAddress("L1_ANCHORING_MANAGER");
        address feeOracle = vm.envAddress("FEE_ORACLE");

        IL1AnchoringManager manager = IL1AnchoringManager(l1AnchoringManager);
        IL1FeeOracle oracle = IL1FeeOracle(feeOracle);

        bytes32 root = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
        uint256 fee = oracle.getFloorFee();
        console.log("Current floor fee:", fee);

        vm.startBroadcast();
        manager.submitForL1Anchoring{value: fee}(root);
        vm.stopBroadcast();
    }
}

contract ConfirmAnchoring is Script {
    function run() public {
        address l1AnchoringManager = vm.envAddress("L1_ANCHORING_GATEWAY");

        IL1AnchoringGateway gateway = IL1AnchoringGateway(l1AnchoringManager);

        bytes32 root = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;

        vm.startBroadcast();
        gateway.submitBatch{value: 0.1 ether}(root, 0, 1);
        vm.stopBroadcast();
    }
}
