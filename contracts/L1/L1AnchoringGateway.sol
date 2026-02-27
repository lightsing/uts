// SPDX-License-Identifier: MIT

pragma solidity ^0.8.24;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {ReentrancyGuardTransient} from "@openzeppelin/contracts/utils/ReentrancyGuardTransient.sol";
import {L1AnchoringGatewayStorage} from "./L1AnchoringGatewayStorage.sol";
import {IL1AnchoringGateway} from "./IL1AnchoringGateway.sol";
import {Constants} from "../Constants.sol";
import {IL1ScrollMessenger} from "scroll-contracts/L1/IL1ScrollMessenger.sol";
import {IL1AnchoringManager} from "../L2/manager/IL1AnchoringManager.sol";

contract L1AnchoringGateway is
    Initializable,
    OwnableUpgradeable,
    UUPSUpgradeable,
    ReentrancyGuardTransient,
    IL1AnchoringGateway
{
    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize(address initialOwner, address l1Messenger, address l1AnchoringManagerL2) public initializer {
        __Ownable_init(initialOwner);

        require(l1Messenger != address(0), "UTS: Invalid L1ScrollMessenger address");
        require(l1AnchoringManagerL2 != address(0), "UTS: Invalid L1AnchoringManagerL2 address");

        L1AnchoringGatewayStorage.Storage storage $ = L1AnchoringGatewayStorage.get();
        $.l1Messenger = IL1ScrollMessenger(l1Messenger);
        $.l1AnchoringManagerL2 = IL1AnchoringManager(l1AnchoringManagerL2);
    }

    /// @inheritdoc IL1AnchoringGateway
    function submitBatch(bytes32 merkleRoot, uint256 startIndex, uint256 count) external payable nonReentrant {
        L1AnchoringGatewayStorage.Storage storage $ = L1AnchoringGatewayStorage.get();

        require(address($.l1Messenger) != address(0), "UTS: L1 Scroll Messenger not set");
        require(address($.l1AnchoringManagerL2) != address(0), "UTS: L1 Anchoring Manager L2 not set");

        Constants.UTS.attest(merkleRoot);

        bytes memory message =
            abi.encodeCall(IL1AnchoringManager.confirmL1AnchoringBatch, (merkleRoot, startIndex, count, block.number));

        $.l1Messenger.sendMessage{value: msg.value}(
            address($.l1AnchoringManagerL2),
            0,
            message,
            1_000_000, // TODO: Estimate proper gas limit
            msg.sender // refund the caller for the gas cost of L2 execution
        );
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
}
