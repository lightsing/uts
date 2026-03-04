// SPDX-License-Identifier: MIT

pragma solidity ^0.8.24;

interface IUniversalTimestamps {
    event Attested(bytes32 indexed root, address indexed sender, uint256 timestamp);

    function attest(bytes32 root) external;

    function timestamp(bytes32 root) external view returns (uint256);
}
