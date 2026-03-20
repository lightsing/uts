// SPDX-License-Identifier: MIT

pragma solidity =0.8.28;

import {INFTGenerator} from "./INFTGenerator.sol";
import {Base64} from "openzeppelin-contracts/contracts/utils/Base64.sol";
import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";
import {DateTimeLib} from "solady/utils/DateTimeLib.sol";
import {Code128CGenerator} from "./Code128C.sol";

contract NFTGenerator is Code128CGenerator, INFTGenerator {
    using Strings for *;

    string private constant SVG_START =
        '<svg xmlns="http://www.w3.org/2000/svg" width="420" height="600"><defs><linearGradient id="cardBg" x1="0%" x2="100%" y1="0%" y2="100%"><stop offset="0%" stop-color="#0f0f0f"/><stop offset="100%"/></linearGradient><linearGradient id="glowLineGrad" x1="0%" x2="100%" y1="0%" y2="0%"><stop offset="0%" stop-color="transparent"/><stop offset="50%" stop-color="#3b82f6"/><stop offset="80%" stop-color="#60a5fa"/><stop offset="100%" stop-color="transparent"/></linearGradient><clipPath id="cardClip"><rect width="420" height="600" rx="28"/></clipPath><clipPath id="topClip"><rect width="420" height="250" rx="28"/></clipPath><radialGradient id="coreBg" cx="50%" cy="50%" r="70%" fx="50%" fy="50%"><stop offset="0%" stop-color="#111827"/><stop offset="100%"/></radialGradient><pattern id="grid" width="30" height="30" patternUnits="userSpaceOnUse"><path fill="none" stroke="rgba(59,130,246,0.05)" d="M30 0H0v30"/></pattern><style>@keyframes flow{0%{transform:translateX(-100%)}to{transform:translateX(100%)}}svg text {font-size-adjust: 0.5}.sans{font-family:system-ui,-apple-system,BlinkMacSystemFont,\'Segoe UI\',Roboto,\'Helvetica Neue\',Arial,\'Noto Sans\',\'PingFang SC\',\'Microsoft YaHei\',sans-serif}.mono{font-family:\'Consolas\',\'Monaco\',\'Microsoft YaHei\',\'PingFang SC\',monospace}.bold{font-weight:700}.c{dominant-baseline:central;}.m{text-anchor:middle;}</style></defs><g clip-path="url(#cardClip)"><path fill="url(#cardBg)" d="M0 0h420v600H0z"/><rect width="420" height="600" fill="none" stroke="rgba(255,255,255,0.08)" stroke-width="2" rx="28"/><g clip-path="url(#topClip)"><path fill="url(#coreBg)" d="M0 0h420v250H0z"/><path fill="url(#grid)" d="M0 0h420v250H0z"/><path fill="url(#glowLineGrad)" d="M0 0h420v2H0z" style="animation:flow 4s infinite linear"/></g><path stroke="rgba(255,255,255,0.05)" d="M0 250h420"/><g transform="translate(110 45)"><path fill="none" stroke="rgba(59, 130, 246, 0.2)" stroke-width="1.5" d="M100 20 50 70m50-50 50 50M50 70l-25 50m25-50 25 50m75-50-25 50m25-50 25 50"/><rect width="16" height="16" x="92" y="12" fill="#3b82f6" rx="3"/><rect width="16" height="16" x="42" y="62" fill="#1d4ed8" opacity=".6" rx="3"/><rect width="16" height="16" x="142" y="62" fill="#1d4ed8" opacity=".6" rx="3"/><g fill="#1e293b" stroke="#3b82f6" opacity=".4"><rect width="16" height="16" x="17" y="112" rx="2"/><rect width="16" height="16" x="67" y="112" rx="2"/><rect width="16" height="16" x="117" y="112" rx="2"/><rect width="16" height="16" x="167" y="112" rx="2"/></g></g><rect width="102" height="22" x="290" y="210" fill="rgba(16,185,129,0.1)" stroke="rgba(52,211,153,0.2)" rx="6"/><rect width="356" height="66" x="32" y="305" fill="rgba(59,130,246,0.05)" stroke="rgba(59,130,246,0.15)" rx="16"/><g fill="rgba(255,255,255,0.03)" stroke="rgba(255,255,255,0.05)"><rect width="170" height="55" x="32" y="415" rx="12"/><rect width="356" height="45" x="32" y="495" rx="12"/><rect width="170" height="55" x="218" y="415" rx="12"/></g><g class="sans bold"><text x="32" y="45" fill="#60a5fa" font-size="10" letter-spacing="1.5" opacity=".7" lengthAdjust="spacing">CERTIFICATE ID</text><g class="c" fill="#475569" font-size="12"><text x="32" y="289" fill="#3b82f6" class="c" font-size="14" textLength="125" lengthAdjust="spacing">CONTENT HASH</text><text x="341" y="221" fill="#34d399" class="m" textLength="70" lengthAdjust="spacing">ANCHORED</text><text x="32" y="405" textLength="140" lengthAdjust="spacing">SCROLL ATTEST HEIGHT</text><text x="218" y="405" textLength="170" lengthAdjust="spacing">ETHEREUM ANCHOR HEIGHT</text><text x="48" y="518" textLength="70" lengthAdjust="spacing">TIMESTAMP</text></g></g><g fill="#eff6ff" class="mono" font-size="18"><g class="bold c"><g text-anchor="start"><text x="30" y="70" font-size="28" letter-spacing="-1">#';

    // forge-lint: disable-next-line(mixed-case-function)
    function generateTokenURI(
        uint256 tokenId,
        bytes32 contentHash,
        uint256 l2BlockNumber,
        uint256 l1BlockNumber,
        uint256 time,
        string memory l2Name
    ) external view override returns (string memory) {
        string memory image = string.concat(
            '"image":"data:image/svg+xml;base64,',
            Base64.encode(bytes(_generateSvg(tokenId, contentHash, l2BlockNumber, l1BlockNumber, time)))
        );
        string memory json = string.concat(
            '{"name":"Certificate #',
            Strings.toString(tokenId),
            " - ",
            l2Name,
            '",',
            '"description":"Certificate representing the content implied by the hash exists after the timestamp.","external_url": "https://timestamps.now/',
            Strings.toString(block.chainid),
            "/",
            Strings.toString(tokenId),
            '",',
            image,
            '","attributes": [{"display_type": "date", "trait_type": "Timestamp", "value": ',
            Strings.toString(time),
            '},{"trait_type": "L1 Block Number", "value": ',
            Strings.toString(l1BlockNumber),
            '},{"trait_type": "L2 Block Number", "value": ',
            Strings.toString(l2BlockNumber),
            "}]}"
        );
        return string.concat("data:application/json;base64,", Base64.encode(bytes(json)));
    }

    // forge-lint: disable-next-line(mixed-case-function)
    function _generateSvg(
        uint256 tokenId,
        bytes32 contentHash,
        uint256 l2BlockNumber,
        uint256 l1BlockNumber,
        uint256 time
    ) internal view virtual returns (bytes memory) {
        string memory contentHashStr = Strings.toHexString(abi.encodePacked(contentHash));
        string memory contentHashPart1 = _subAsciiString(contentHashStr, 2, 34);
        string memory contentHashPart2 = _subAsciiString(contentHashStr, 34, 66);

        return abi.encodePacked(
            SVG_START,
            _formatWithCommas(tokenId),
            "</text>",
            '<text x="48" y="442">',
            _formatWithCommas(l2BlockNumber),
            "</text>",
            '<text x="234" y="442">',
            _formatWithCommas(l1BlockNumber),
            "</text>",
            "</g>",
            '<g class="m">',
            '<text x="210" y="329" textLength="310" lengthAdjust="spacing">',
            contentHashPart1,
            "</text>",
            '<text x="210" y="346" textLength="310" lengthAdjust="spacing">',
            contentHashPart2,
            "</text>",
            "</g>",
            '<text x="372" y="518" fill="#60a5fa" class="mono bold c" font-size="14" text-anchor="end">',
            _formatTimestamp(time),
            "</text>",
            "</g>",
            '<text x="388" y="570" font-size="9" opacity=".3" text-anchor="end" textLength="200" lengthAdjust="spacing">UNIVERSAL TIMESTAMPS PROTOCOL</text>',
            "</g>",
            '<g opacity="0.5" transform="translate(32, 558)">',
            generateTokenBarcodeSvg(tokenId),
            "</g>",
            "</g>",
            "</svg>"
        );
    }

    function _formatTimestamp(uint256 timestamp) internal pure returns (string memory) {
        (uint256 year, uint256 month, uint256 day, uint256 hour, uint256 minute, uint256 second) =
            DateTimeLib.timestampToDateTime(timestamp);

        return string.concat(
            Strings.toString(year),
            "-",
            _padZero(month),
            "-",
            _padZero(day),
            " ",
            _padZero(hour),
            ":",
            _padZero(minute),
            ":",
            _padZero(second)
        );
    }

    function _subAsciiString(string memory str, uint256 startIndex, uint256 endIndex)
        internal
        pure
        returns (string memory)
    {
        bytes memory strBytes = bytes(str);
        require(startIndex <= endIndex && endIndex <= strBytes.length);

        bytes memory result = new bytes(endIndex - startIndex);
        for (uint256 i = startIndex; i < endIndex; i++) {
            result[i - startIndex] = strBytes[i];
        }
        return string(result);
    }

    function _padZero(uint256 value) internal pure returns (string memory) {
        if (value < 10) {
            return string.concat("0", Strings.toString(value));
        }
        return Strings.toString(value);
    }

    function _formatWithCommas(uint256 value) internal pure returns (string memory) {
        if (value == 0) return "0";

        bytes memory strBytes = bytes(Strings.toString(value));
        uint256 len = strBytes.length;

        uint256 commaCount = (len - 1) / 3;

        bytes memory result = new bytes(len + commaCount);

        uint256 j = result.length;
        for (uint256 i = len; i > 0; i--) {
            j--;
            result[j] = strBytes[i - 1];

            if ((len - i + 1) % 3 == 0 && i > 1) {
                j--;
                // casting to 'bytes1' is safe because ',' is a valid ASCII character
                // forge-lint: disable-next-line(unsafe-typecast)
                result[j] = bytes1(",");
            }
        }

        return string(result);
    }
}
