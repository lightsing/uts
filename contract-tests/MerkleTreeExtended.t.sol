// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

import {Test} from "forge-std/Test.sol";
import {MerkleTree} from "../contracts/core/MerkleTree.sol";

contract MerkleTreeWrapper {
    function computeRoot(bytes32[] memory leaves) external pure returns (bytes32) {
        return MerkleTree.computeRoot(leaves);
    }

    function verify(bytes32[] memory leaves, bytes32 expectedRoot) external pure returns (bool) {
        return MerkleTree.verify(leaves, expectedRoot);
    }
}

contract MerkleTreeExtendedTest is Test {
    MerkleTreeWrapper wrapper;

    function setUp() public {
        wrapper = new MerkleTreeWrapper();
    }

    function test_ComputeRoot_SingleLeaf() public pure {
        bytes32[] memory leaves = new bytes32[](1);
        leaves[0] = keccak256("single");
        bytes32 root = MerkleTree.computeRoot(leaves);
        assertEq(root, leaves[0], "Single leaf should return the leaf itself");
    }

    function test_ComputeRoot_TwoLeaves() public pure {
        bytes32[] memory leaves = new bytes32[](2);
        leaves[0] = keccak256(abi.encodePacked(uint256(0)));
        leaves[1] = keccak256(abi.encodePacked(uint256(1)));
        bytes32 root = MerkleTree.computeRoot(leaves);
        bytes32 expected = MerkleTree.hashNode(leaves[0], leaves[1]);
        assertEq(root, expected, "Two leaves root mismatch");
    }

    function test_ComputeRoot_ThreeLeaves() public pure {
        bytes32[] memory leaves = new bytes32[](3);
        leaves[0] = keccak256(abi.encodePacked(uint256(10)));
        leaves[1] = keccak256(abi.encodePacked(uint256(20)));
        leaves[2] = keccak256(abi.encodePacked(uint256(30)));
        bytes32 root = MerkleTree.computeRoot(leaves);

        // Manually compute: pad to 4 leaves
        bytes32 h01 = MerkleTree.hashNode(leaves[0], leaves[1]);
        bytes32 h23 = MerkleTree.hashNode(leaves[2], bytes32(0));
        bytes32 expected = MerkleTree.hashNode(h01, h23);
        assertEq(root, expected, "Three leaves root mismatch");
    }

    function test_ComputeRoot_FiveLeaves() public pure {
        bytes32[] memory leaves = new bytes32[](5);
        for (uint256 i = 0; i < 5; i++) {
            leaves[i] = keccak256(abi.encodePacked(i + 100));
        }
        bytes32 root = MerkleTree.computeRoot(leaves);

        // Manually compute: pad to 8 leaves (next power of 2)
        bytes32 h01 = MerkleTree.hashNode(leaves[0], leaves[1]);
        bytes32 h23 = MerkleTree.hashNode(leaves[2], leaves[3]);
        bytes32 h45 = MerkleTree.hashNode(leaves[4], bytes32(0));
        bytes32 h67 = MerkleTree.hashNode(bytes32(0), bytes32(0));
        bytes32 h0123 = MerkleTree.hashNode(h01, h23);
        bytes32 h4567 = MerkleTree.hashNode(h45, h67);
        bytes32 expected = MerkleTree.hashNode(h0123, h4567);
        assertEq(root, expected, "Five leaves root mismatch");
    }

    function test_ComputeRoot_EmptyArray_Reverts() public {
        bytes32[] memory leaves = new bytes32[](0);
        vm.expectRevert("Merkle: Cannot compute root of empty set");
        wrapper.computeRoot(leaves);
    }

    function test_Verify_ValidRoot() public pure {
        bytes32[] memory leaves = new bytes32[](3);
        leaves[0] = keccak256("a");
        leaves[1] = keccak256("b");
        leaves[2] = keccak256("c");
        bytes32 root = MerkleTree.computeRoot(leaves);
        assertTrue(MerkleTree.verify(leaves, root), "Valid root should verify");
    }

    function test_Verify_InvalidRoot() public pure {
        bytes32[] memory leaves = new bytes32[](3);
        leaves[0] = keccak256("a");
        leaves[1] = keccak256("b");
        leaves[2] = keccak256("c");
        assertFalse(MerkleTree.verify(leaves, keccak256("wrong")), "Invalid root should not verify");
    }

    function testFuzz_nextPowerOfTwo(uint256 n) public pure {
        n = bound(n, 0, 2 ** 128);
        uint256 result = MerkleTree.nextPowerOfTwo(n);
        assertTrue(result >= n, "nextPowerOfTwo should be >= n");
        // result should be a power of 2 (or 1 for n=0)
        if (result > 0) {
            assertEq(result & (result - 1), 0, "Result should be a power of two");
        }
    }

    function testFuzz_ComputeRoot_Deterministic(bytes32 seed) public pure {
        uint256 count = (uint256(seed) % 10) + 1; // 1 to 10 leaves
        bytes32[] memory leaves = new bytes32[](count);
        for (uint256 i = 0; i < count; i++) {
            leaves[i] = keccak256(abi.encodePacked(seed, i));
        }
        bytes32 root1 = MerkleTree.computeRoot(leaves);
        bytes32 root2 = MerkleTree.computeRoot(leaves);
        assertEq(root1, root2, "Same input should produce same root");
    }

    function test_HashNode_Symmetry() public pure {
        bytes32 a = keccak256("left");
        bytes32 b = keccak256("right");
        bytes32 ab = MerkleTree.hashNode(a, b);
        bytes32 ba = MerkleTree.hashNode(b, a);
        assertTrue(ab != ba, "hashNode(a,b) should differ from hashNode(b,a)");
    }
}
