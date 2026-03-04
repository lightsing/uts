// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test, console} from "forge-std/Test.sol";
import {UniversalTimestamps} from "../contracts/UniversalTimestamps.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

contract UniversalTimestampsV2 is UniversalTimestamps {
    function version() public pure returns (string memory) {
        return "V2";
    }
}

contract UniversalTimestampsTest is Test {
    UniversalTimestamps public proxy;
    address owner = address(1);

    function setUp() public {
        UniversalTimestamps implementation = new UniversalTimestamps();

        bytes memory initData = abi.encodeWithSelector(UniversalTimestamps.initialize.selector, owner);
        ERC1967Proxy proxyAddress = new ERC1967Proxy(address(implementation), initData);

        proxy = UniversalTimestamps(address(proxyAddress));
    }

    function test_StoragePersistenceAfterUpgrade() public {
        bytes32 root = keccak256("test_data");

        proxy.attest(root);
        uint256 timeV1 = proxy.timestamp(root);
        assertGt(timeV1, 0);

        vm.startPrank(owner);

        UniversalTimestampsV2 v2Impl = new UniversalTimestampsV2();

        proxy.upgradeToAndCall(address(v2Impl), "");

        vm.stopPrank();

        UniversalTimestampsV2 proxyV2 = UniversalTimestampsV2(address(proxy));

        assertEq(proxyV2.version(), "V2");

        assertEq(proxyV2.timestamp(root), timeV1);
        console.log("Storage persisted across upgrade at namespaced slot.");
    }
}
