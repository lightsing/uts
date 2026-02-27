// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import {Script, console} from "forge-std/Script.sol";
import {IL2AnchoringManager} from "../contracts/L2/manager/IL2AnchoringManager.sol";
import {IL1AnchoringGateway} from "../contracts/L1/IL1AnchoringGateway.sol";
import {IL1FeeOracle} from "../contracts/L2/oracle/IL1FeeOracle.sol";
import {MerkleTree} from "../contracts/core/MerkleTree.sol";

contract SubmitAnchoring is Script {
    function run() public {
        address anchoringManager = vm.envAddress("ANCHORING_MANAGER");
        address feeOracle = vm.envAddress("FEE_ORACLE");

        IL2AnchoringManager manager = IL2AnchoringManager(anchoringManager);
        IL1FeeOracle oracle = IL1FeeOracle(feeOracle);

        bytes32 root = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
        uint256 fee = oracle.getFloorFee();
        console.log("Current floor fee:", fee);

        vm.startBroadcast();
        manager.submitForL1Anchoring{value: fee}(root);
        vm.stopBroadcast();
    }
}

contract SubmitAnchoring2 is Script {
    function run() public {
        address anchoringManager = vm.envAddress("ANCHORING_MANAGER");
        address feeOracle = vm.envAddress("FEE_ORACLE");

        IL2AnchoringManager manager = IL2AnchoringManager(anchoringManager);
        IL1FeeOracle oracle = IL1FeeOracle(feeOracle);

        bytes32 root = 0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd;
        uint256 fee = oracle.getFloorFee();
        console.log("Current floor fee:", fee);

        vm.startBroadcast();
        manager.submitForL1Anchoring{value: fee}(root);
        vm.stopBroadcast();
    }
}

contract ConfirmAnchoring is Script {
    function run() public {
        address anchoringGateway = vm.envAddress("ANCHORING_GATEWAY");

        IL1AnchoringGateway gateway = IL1AnchoringGateway(anchoringGateway);

        bytes32[] memory roots = new bytes32[](2);
        roots[0] = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
        roots[1] = 0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd;

        bytes32 root = MerkleTree.computeRoot(roots);

        vm.startBroadcast();
        gateway.submitBatch{value: 0.1 ether}(root, 0, 2);
        vm.stopBroadcast();
    }
}
