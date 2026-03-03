// SPDX-License-Identifier: MIT

pragma solidity =0.8.28;

interface INFTGenerator {
    function generateTokenURI(
        uint256 tokenId,
        bytes32 contentHash,
        uint256 l2BlockNumber,
        uint256 l1BlockNumber,
        uint256 time,
        string memory l2Name
    ) external view returns (string memory);
}
