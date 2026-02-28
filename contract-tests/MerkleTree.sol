// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import {Test, console} from "forge-std/Test.sol";
import {MerkleTree} from "../contracts/core/MerkleTree.sol";

/**
 * @title MerkleTreeTest
 * @dev Tests to verify the correctness of the MerkleTree library functions
 */
contract MerkleTreeTest is Test {
    function test_nextPowerOfTwo() public pure {
        assertEq(MerkleTree.nextPowerOfTwo(0), 1);
        assertEq(MerkleTree.nextPowerOfTwo(1), 1);
        assertEq(MerkleTree.nextPowerOfTwo(2), 2);
        assertEq(MerkleTree.nextPowerOfTwo(3), 4);
        assertEq(MerkleTree.nextPowerOfTwo(4), 4);
        assertEq(MerkleTree.nextPowerOfTwo(5), 8);
        assertEq(MerkleTree.nextPowerOfTwo(6), 8);
        assertEq(MerkleTree.nextPowerOfTwo(7), 8);
        assertEq(MerkleTree.nextPowerOfTwo(8), 8);
        assertEq(MerkleTree.nextPowerOfTwo(9), 16);
        assertEq(MerkleTree.nextPowerOfTwo(15), 16);
        assertEq(MerkleTree.nextPowerOfTwo(16), 16);
        assertEq(MerkleTree.nextPowerOfTwo(17), 32);
    }

    function test_hashNode() public pure {
        // Test with known values
        bytes32 left = keccak256(abi.encodePacked(uint256(0)));
        assertEq(left, bytes32(0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563));
        bytes32 right = keccak256(abi.encodePacked(uint256(1)));
        assertEq(right, bytes32(0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6));
        bytes32 expectedHash = bytes32(0x05cca086ac1292d712fa72c1b3f12ca115644d2961f946ba815d8d731f9e5059);
        assertEq(MerkleTree.hashNode(left, right), expectedHash);
    }

    function test_Max1024Leaves() public pure {
        bytes32[] memory leaves = new bytes32[](1024);
        for (uint256 i = 0; i < 1024; i++) {
            leaves[i] = keccak256(abi.encodePacked(i));
        }
        bytes32 root = MerkleTree.computeRoot(leaves);
        bytes32 expected = bytes32(0x168852131b9462b8137b1deadf05872035b6046281ecf234278545aef52ae47b);
        assertEq(root, expected, "Root mismatch for 1024 leaves");
    }
}
