// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";

contract Code128CGenerator {
    function _getCode128Widths(uint256 index) internal pure returns (uint32) {
        uint32[107] memory W = [
            uint32(0x212222),
            0x222122,
            0x222221,
            0x121223,
            0x121322,
            0x131222,
            0x122213,
            0x122312,
            0x132212,
            0x221213,
            0x221312,
            0x231212,
            0x112232,
            0x122132,
            0x122231,
            0x113222,
            0x123122,
            0x123221,
            0x223211,
            0x221132,
            0x221231,
            0x213212,
            0x223112,
            0x312131,
            0x311222,
            0x321122,
            0x321221,
            0x312212,
            0x322112,
            0x322211,
            0x212123,
            0x212321,
            0x232121,
            0x111323,
            0x131123,
            0x131321,
            0x112313,
            0x132113,
            0x132311,
            0x211313,
            0x231113,
            0x231311,
            0x112133,
            0x112331,
            0x132131,
            0x113123,
            0x113321,
            0x133121,
            0x313121,
            0x211331,
            0x231131,
            0x213113,
            0x213311,
            0x213131,
            0x311123,
            0x311321,
            0x331121,
            0x312113,
            0x312311,
            0x332111,
            0x314111,
            0x221411,
            0x431111,
            0x111224,
            0x111422,
            0x121124,
            0x121421,
            0x141122,
            0x141221,
            0x112214,
            0x112412,
            0x122114,
            0x122411,
            0x142112,
            0x142211,
            0x241211,
            0x221114,
            0x413111,
            0x241112,
            0x134111,
            0x111242,
            0x121142,
            0x121241,
            0x114212,
            0x124112,
            0x124211,
            0x411212,
            0x421112,
            0x421211,
            0x212141,
            0x214121,
            0x412121,
            0x111143,
            0x111341,
            0x131141,
            0x114113,
            0x114311,
            0x411113,
            0x411311,
            0x113141,
            0x114131,
            0x311141,
            0x411131,
            0x211412,
            0x211214,
            0x211232,
            0x2331112
        ];
        return W[index];
    }

    /// @dev pad tokenId to 20 digits
    function formatPaddedTokenId(uint256 tokenId) internal pure returns (string memory) {
        bytes memory padded = new bytes(20);
        uint256 temp = tokenId;
        for (uint256 i = 20; i > 0; i--) {
            // ascii '0' = 48
            padded[i - 1] = bytes1(uint8(48 + (temp % 10)));
            temp /= 10;
        }
        return string(padded);
    }

    /// @dev Code 128-C
    function generateTokenBarcodeSvg(uint256 tokenId) public pure returns (string memory) {
        uint256[] memory values = new uint256[](13);
        values[0] = 105; // Start C

        uint256 temp = tokenId;
        uint256 checksum = 105;

        for (uint256 i = 10; i >= 1; i--) {
            uint256 pair = temp % 100;
            values[i] = pair;
            temp /= 100;
        }

        //  Checksum
        for (uint256 i = 1; i <= 10; i++) {
            checksum += (i * values[i]);
        }
        values[11] = checksum % 103;
        values[12] = 106; // Stop

        bytes memory dWhite;
        bytes memory dBlue;

        uint256 xPos = 0;
        uint256 moduleWidth = 1;
        bool usingWhite = true;

        for (uint256 i = 0; i < 13; i++) {
            if (usingWhite && i != 0 && values[i] != 0) {
                usingWhite = false;
            }
            if (i == 12) {
                usingWhite = true;
            }

            uint32 pattern = _getCode128Widths(values[i]);
            bool isBar = true;

            for (uint256 j = 0; j < 7; j++) {
                uint256 w = (pattern >> (24 - j * 4)) & 0xF;

                if (w == 0) continue;

                if (isBar) {
                    bytes memory pathSegment =
                        abi.encodePacked("M", Strings.toString(xPos), " 7h", Strings.toString(w * moduleWidth));

                    if (usingWhite) {
                        dWhite = abi.encodePacked(dWhite, pathSegment);
                    } else {
                        dBlue = abi.encodePacked(dBlue, pathSegment);
                    }
                }

                xPos += w * moduleWidth;
                isBar = !isBar;
            }
        }

        return string(
            abi.encodePacked(
                '<g stroke-width="14" fill="none">',
                '<path stroke="#fff" d="',
                dWhite,
                '"/>',
                '<path stroke="#3b82f6" d="',
                dBlue,
                '"/>',
                "</g>"
            )
        );
    }
}
