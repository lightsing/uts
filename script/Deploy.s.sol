// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import {Script, console} from "forge-std/Script.sol";
import {UniversalTimestamps} from "../contracts/core/UniversalTimestamps.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {L2AnchoringManager} from "../contracts/L2/manager/L2AnchoringManager.sol";
import {IL2AnchoringManager} from "../contracts/L2/manager/IL2AnchoringManager.sol";
import {L1FeeOracle} from "../contracts/L2/oracle/L1FeeOracle.sol";
import {IL1FeeOracle} from "../contracts/L2/oracle/IL1FeeOracle.sol";
import {L1AnchoringGateway} from "../contracts/L1/L1AnchoringGateway.sol";

bytes32 constant SALT = keccak256("UniversalTimestamps");

contract DeployTimestampCreate2 is Script {
    function run() public {
        address owner = vm.envAddress("OWNER_ADDRESS");

        vm.startBroadcast();
        UniversalTimestamps uts = new UniversalTimestamps{salt: SALT}();
        vm.stopBroadcast();
        console.log("UTS deployed at:", address(uts));
    }
}

contract DeployFeeOracle is Script {
    function run() public {
        address owner = vm.envAddress("OWNER_ADDRESS");

        vm.startBroadcast();
        L1FeeOracle implementation = new L1FeeOracle(
            owner,
            100_000, // initialGasPerAttestation
            0.5e18 // initialDiscountRatio (50%)
        );
        vm.stopBroadcast();

        console.log("FeeOracle deployed at", address(implementation));
    }
}

contract DeployManager is Script {
    function run() public {
        address owner = vm.envAddress("OWNER_ADDRESS");
        address uts = vm.envAddress("UTS");
        address feeOracle = vm.envAddress("FEE_ORACLE");
        address l1Messenger = vm.envAddress("L1_MESSENGER");
        address l2Messenger = vm.envAddress("L2_MESSENGER");

        vm.startBroadcast();
        L2AnchoringManager implementation = new L2AnchoringManager();
        console.log("Implementation deployed at:", address(implementation));
        // function initialize(address initialOwner, address uts, address feeOracle, address l1Messenger, address l2Messenger)
        bytes memory initData =
            abi.encodeCall(L2AnchoringManager.initialize, (owner, uts, feeOracle, l1Messenger, l2Messenger));

        ERC1967Proxy proxy = new ERC1967Proxy{salt: SALT}(address(implementation), initData);
        vm.stopBroadcast();

        console.log("Proxy deployed at:", address(proxy));
    }
}

contract DeployGateway is Script {
    function run() public {
        address owner = vm.envAddress("OWNER_ADDRESS");
        address uts = vm.envAddress("UTS");
        address l1Messenger = vm.envAddress("L1_MESSENGER");
        address anchoringManager = vm.envAddress("ANCHORING_MANAGER");

        vm.startBroadcast();
        L1AnchoringGateway implementation = new L1AnchoringGateway();
        console.log("Implementation deployed at:", address(implementation));
        // function initialize(address initialOwner, address uts, address l1Messenger, address l2AnchoringManager)
        bytes memory initData =
            abi.encodeCall(L1AnchoringGateway.initialize, (owner, uts, l1Messenger, anchoringManager));

        ERC1967Proxy proxy = new ERC1967Proxy{salt: SALT}(address(implementation), initData);
        vm.stopBroadcast();

        console.log("Proxy deployed at:", address(proxy));
    }
}

contract SetGateway is Script {
    function run() public {
        address l1Gateway = vm.envAddress("ANCHORING_GATEWAY");
        address anchoringManager = vm.envAddress("ANCHORING_MANAGER");

        IL2AnchoringManager manager = IL2AnchoringManager(anchoringManager);
        vm.startBroadcast();
        manager.setL1Gateway(l1Gateway);
        vm.stopBroadcast();
    }
}

contract UpgradeManager is Script {
    function run() public {
        address anchoringManager = vm.envAddress("ANCHORING_MANAGER");

        vm.startBroadcast();
        L2AnchoringManager newImplementation = new L2AnchoringManager();
        console.log("New implementation deployed at:", address(newImplementation));

        L2AnchoringManager manager = L2AnchoringManager(payable(anchoringManager));

        manager.upgradeToAndCall(address(newImplementation), "");
        vm.stopBroadcast();

        console.log("Manager upgraded to new implementation at:", address(newImplementation));
    }
}

contract UpgradeGateway is Script {
    function run() public {
        address l1Gateway = vm.envAddress("ANCHORING_GATEWAY");

        vm.startBroadcast();
        L1AnchoringGateway newImplementation = new L1AnchoringGateway();
        console.log("New implementation deployed at:", address(newImplementation));

        L1AnchoringGateway gateway = L1AnchoringGateway(payable(l1Gateway));

        gateway.upgradeToAndCall(address(newImplementation), "");
        vm.stopBroadcast();

        console.log("Gateway upgraded to new implementation at:", address(newImplementation));
    }
}
