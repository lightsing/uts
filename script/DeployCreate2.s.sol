// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {UniversalTimestamps} from "../contracts/UniversalTimestamps.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

interface ICreateX {
    function deployCreate2(bytes32 salt, bytes memory initCode) external payable returns (address);

    function computeCreate2Address(bytes32 salt, bytes32 initCodeHash) external view returns (address);
}

contract DeployCreate2 is Script {
    // CreateX is deployed at the same address on all supported chains
    ICreateX constant CREATEX = ICreateX(0xba5Ed099633D3B313e4D5F7bdc1305d3c28ba5Ed);
    bytes32 constant SALT = keccak256("universal-timestamps");

    function run() public {
        address owner = vm.envAddress("OWNER_ADDRESS");

        vm.startBroadcast();
        UniversalTimestamps implementation = new UniversalTimestamps{salt: SALT}();
        //   Implementation deployed at: 0x2D806e4ae1c3FDCfecb019B192a53371CAC889A7
        console.log("Implementation deployed at:", address(implementation));

        bytes memory initData = abi.encodeCall(UniversalTimestamps.initialize, (owner));

        ERC1967Proxy proxy = new ERC1967Proxy{salt: SALT}(address(implementation), initData);
        vm.stopBroadcast();

        //   Proxy deployed at: 0xceB7a9E77bd00D0391349B9bC989167cAB5e35e7
        console.log("Proxy deployed at:", address(proxy));
    }
}
