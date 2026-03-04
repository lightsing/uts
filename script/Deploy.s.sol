// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {L2AnchoringManager} from "../contracts/L2/manager/L2AnchoringManager.sol";
import {IL2AnchoringManager} from "../contracts/L2/manager/IL2AnchoringManager.sol";
import {FeeOracle} from "../contracts/L2/oracle/FeeOracle.sol";
import {IFeeOracle} from "../contracts/L2/oracle/IFeeOracle.sol";
import {L1AnchoringGateway} from "../contracts/L1/L1AnchoringGateway.sol";
import {NFTGenerator} from "../contracts/L2/nft/NFTGenerator.sol";

bytes32 constant SALT = keccak256("UniversalTimestamps");

// Deployment requires to run the scripts in the order they are defined in this file.

// Verify Args:
// --verifier etherscan --verifier-url "https://api.etherscan.io/v2/api?chainid={chain_id}" --etherscan-api-key {key} --verify --retries 10 --delay 10

// forge script script/Deploy.s.sol:DeployFeeOracle --broadcast --rpc-url https://sepolia-rpc.scroll.io/ --account dev {verify_args}
contract DeployFeeOracle is Script {
    function run() public {
        address owner = vm.envAddress("OWNER_ADDRESS");

        vm.startBroadcast();
        FeeOracle implementation = new FeeOracle(owner);
        vm.stopBroadcast();

        console.log("FeeOracle deployed at", address(implementation));
    }
}

// forge script script/Deploy.s.sol:DeployNFTGenerator --broadcast --rpc-url https://sepolia-rpc.scroll.io/ --account dev {verify_args}
contract DeployNFTGenerator is Script {
    function run() public {
        vm.startBroadcast();
        NFTGenerator generator = new NFTGenerator();
        vm.stopBroadcast();

        console.log("NFTGenerator deployed at", address(generator));
    }
}

// This should deploy using master account for deterministic address.
// forge script script/Deploy.s.sol:DeployManager --broadcast --rpc-url https://sepolia-rpc.scroll.io/ --account master --sender $MASTER_ADDRESS {verify_args}
contract DeployManager is Script {
    function run() public {
        address master = vm.envAddress("MASTER_ADDRESS");
        address owner = vm.envAddress("OWNER_ADDRESS");
        address eas = vm.envAddress("EAS_SCROLL");
        address feeOracle = vm.envAddress("FEE_ORACLE");
        address generator = vm.envAddress("NFT_GENERATOR");
        address l2Messenger = vm.envAddress("L2_MESSENGER");

        vm.startBroadcast(master);
        L2AnchoringManager implementation = new L2AnchoringManager{salt: SALT}();
        console.log("Implementation deployed at:", address(implementation));

        bytes memory initData = abi.encodeCall(L2AnchoringManager.initialize, (master));
        ERC1967Proxy proxy = new ERC1967Proxy{salt: SALT}(address(implementation), initData);

        L2AnchoringManager manager = L2AnchoringManager(payable(address(proxy)));
        manager.lateInitialize("Scroll", owner, eas, feeOracle, l2Messenger, generator);
        vm.stopBroadcast();

        console.log("Proxy deployed at:", address(proxy));
    }
}

// forge script script/Deploy.s.sol:AcceptManagerAdmin --broadcast --rpc-url https://sepolia-rpc.scroll.io/
contract AcceptManagerAdmin is Script {
    function run() public {
        address anchoringManager = vm.envAddress("ANCHORING_MANAGER");

        L2AnchoringManager manager = L2AnchoringManager(payable(anchoringManager));

        vm.startBroadcast();
        manager.completeInitialization();
        vm.stopBroadcast();
    }
}

// This should deploy using master account for deterministic address.
// forge script script/Deploy.s.sol:DeployGateway --broadcast --rpc-url https://0xrpc.io/sep --account master --sender $MASTER_ADDRESS {verify_args}
contract DeployGateway is Script {
    function run() public {
        address master = vm.envAddress("MASTER_ADDRESS");
        address owner = vm.envAddress("OWNER_ADDRESS");
        address eas = vm.envAddress("EAS_MAINNET");
        address l1Messenger = vm.envAddress("L1_MESSENGER");
        address anchoringManager = vm.envAddress("ANCHORING_MANAGER");

        vm.startBroadcast(master);
        L1AnchoringGateway implementation = new L1AnchoringGateway{salt: SALT}();
        console.log("Implementation deployed at:", address(implementation));

        bytes memory initData = abi.encodeCall(L1AnchoringGateway.initialize, (master));
        ERC1967Proxy proxy = new ERC1967Proxy{salt: SALT}(address(implementation), initData);

        L1AnchoringGateway gateway = L1AnchoringGateway(payable(address(proxy)));
        gateway.lateInitialize(owner, eas, l1Messenger, anchoringManager);
        vm.stopBroadcast();

        console.log("Proxy deployed at:", address(proxy));
    }
}

// forge script script/Deploy.s.sol:AcceptGatewayAdmin --broadcast --rpc-url https://0xrpc.io/sep
contract AcceptGatewayAdmin is Script {
    function run() public {
        address anchoringGateway = vm.envAddress("ANCHORING_GATEWAY");

        L1AnchoringGateway gateway = L1AnchoringGateway(payable(anchoringGateway));

        vm.startBroadcast();
        gateway.completeInitialization();
        vm.stopBroadcast();
    }
}

// forge script script/Deploy.s.sol:SetGateway --broadcast --rpc-url https://sepolia-rpc.scroll.io/ --account dev {verify_args}
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

// --- Following scripts are utils, and should be run only when needed. ---

contract GrantSubmitter is Script {
    function run() public {
        address anchoringGateway = vm.envAddress("ANCHORING_GATEWAY");
        address submitter = vm.envAddress("SUBMITTER");

        L1AnchoringGateway gateway = L1AnchoringGateway(payable(anchoringGateway));

        vm.startBroadcast();
        gateway.grantRole(gateway.SUBMITTER_ROLE(), submitter);
        vm.stopBroadcast();

        console.log("Granted SUBMITTER_ROLE to", submitter);
    }
}

contract SetManager is Script {
    function run() public {
        address gateway = vm.envAddress("ANCHORING_GATEWAY");
        address anchoringManager = vm.envAddress("ANCHORING_MANAGER");

        L1AnchoringGateway l1Gateway = L1AnchoringGateway(payable(gateway));
        vm.startBroadcast();
        l1Gateway.setL2AnchoringManager(anchoringManager);
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

contract ClearBatch is Script {
    function run() public {
        address anchoringManager = vm.envAddress("ANCHORING_MANAGER");

        IL2AnchoringManager manager = IL2AnchoringManager(anchoringManager);

        vm.startBroadcast();
        manager.clearBatch();
        vm.stopBroadcast();
    }
}
