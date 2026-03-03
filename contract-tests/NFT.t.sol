// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {NFTGenerator} from "../contracts/L2/nft/NFTGenerator.sol";

contract NFTGeneratorHarness is NFTGenerator {
    function exposedGenerateSvg(
        uint256 tokenId,
        bytes32 contentHash,
        uint256 l2BlockNumber,
        uint256 l1BlockNumber,
        uint256 time
    ) external view returns (bytes memory) {
        return _generateSvg(tokenId, contentHash, l2BlockNumber, l1BlockNumber, time);
    }
}

contract NFTGeneratorTest is Test {
    NFTGeneratorHarness generator;

    function setUp() public {
        generator = new NFTGeneratorHarness();
    }

    function testGenerateSVG() public {
        uint256 tokenId = 42424242;
        bytes32 mockedContentHash = keccak256(abi.encodePacked("mock content hash"));
        uint256 mockL2Block = 31087198;
        uint256 mockL1Block = 24570100;
        uint256 mockTime = 1772458623;

        bytes memory rawSvg =
            generator.exposedGenerateSvg(tokenId, mockedContentHash, mockL2Block, mockL1Block, mockTime);

        vm.writeFileBinary("target/foundry/preview.svg", rawSvg);

        string memory fullURI =
            generator.generateTokenURI(tokenId, mockedContentHash, mockL2Block, mockL1Block, mockTime, "Scroll");

        console.log("=== Copy and paste into browser URL bar ===");
        console.log(fullURI);
    }
}
