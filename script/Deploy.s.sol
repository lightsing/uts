// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Script, console} from "forge-std/Script.sol";
import {UniversalTimestamps} from "../contracts/core/UniversalTimestamps.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {L1AnchoringManager} from "../contracts/L2/manager/L1AnchoringManager.sol";
import {L1FeeOracle} from "../contracts/L2/oracle/L1FeeOracle.sol";
import {IL1FeeOracle} from "../contracts/L2/oracle/IL1FeeOracle.sol";
import {L1AnchoringGateway} from "../contracts/L1/L1AnchoringGateway.sol";
import {Constants} from "../contracts/Constants.sol";

bytes32 constant SALT = keccak256("universal-timestamps");

// Note: all address here are sepolia/scroll-sepolia addresses.

contract DeployTimestampCreate2 is Script {
    function run() public {
        address owner = vm.envAddress("OWNER_ADDRESS");

        vm.startBroadcast();
        UniversalTimestamps implementation = new UniversalTimestamps{salt: SALT}();
        //   Implementation deployed at: 0x13889107F758b4c220E6422ef0f00965D5D2b178
        console.log("Implementation deployed at:", address(implementation));

        bytes memory initData = abi.encodeCall(UniversalTimestamps.initialize, (owner));

        ERC1967Proxy proxy = new ERC1967Proxy{salt: SALT}(address(implementation), initData);
        vm.stopBroadcast();

        //   Proxy deployed at: 0xdf939C24d9c075862837e3c9EC0cc1feD6376D59
        console.log("Proxy deployed at:", address(proxy));
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

        //   Proxy deployed at: 0x36E62Da7f040fC19B857541474D2c5dc114f12af
        console.log("FeeOracle deployed at", address(implementation));
    }
}

contract DeployL1Manager is Script {
    function run() public {
        address owner = vm.envAddress("OWNER_ADDRESS");
        address l1Messenger = vm.envAddress("L1_MESSENGER_ADDRESS");

        vm.startBroadcast();
        L1AnchoringManager implementation = new L1AnchoringManager();
        //  Implementation deployed at: 0xc496516540367Aa3E5c209E36d68AD326566943B
        console.log("Implementation deployed at:", address(implementation));

        bytes memory initData = abi.encodeCall(L1AnchoringManager.initialize, (owner, IL1FeeOracle(l1Messenger)));

        ERC1967Proxy proxy = new ERC1967Proxy{salt: SALT}(address(implementation), initData);
        vm.stopBroadcast();

        //   Proxy deployed at: 0x5f44B75D6A0D26533EAECaAcf81eDc9A947a39e9
        console.log("Proxy deployed at:", address(proxy));
    }
}

contract DeployL1Gateway is Script {
    function run() public {
        address owner = vm.envAddress("OWNER_ADDRESS");
        address l1Messenger = vm.envAddress("L1_MESSENGER_ADDRESS");
        address l1AnchoringManagerL2 = vm.envAddress("L1_ANCHORING_MANAGER_L2_ADDRESS");

        vm.startBroadcast();
        L1AnchoringGateway implementation = new L1AnchoringGateway();
        //   Implementation deployed at: 0x8b28f0D465EC9780459E08827E662e35F24D5197
        console.log("Implementation deployed at:", address(implementation));

        bytes memory initData =
            abi.encodeCall(L1AnchoringGateway.initialize, (owner, l1Messenger, l1AnchoringManagerL2));

        ERC1967Proxy proxy = new ERC1967Proxy{salt: SALT}(address(implementation), initData);
        vm.stopBroadcast();

        //   Proxy deployed at: 0x91FA317cf93AAefc044B59df5dd463F513Cea516
        console.log("Proxy deployed at:", address(proxy));
    }
}
