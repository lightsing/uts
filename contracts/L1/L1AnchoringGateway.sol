// SPDX-License-Identifier: MIT

pragma solidity ^0.8.29;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {ReentrancyGuardTransient} from "@openzeppelin/contracts/utils/ReentrancyGuardTransient.sol";
import {L1AnchoringGatewayStorage} from "./L1AnchoringGatewayStorage.sol";
import {IL1AnchoringGateway} from "./IL1AnchoringGateway.sol";
import {IL1ScrollMessenger} from "scroll-contracts/L1/IL1ScrollMessenger.sol";
import {IL2AnchoringManager} from "../L2/manager/IL2AnchoringManager.sol";
import {IUniversalTimestamps} from "../core/IUniversalTimestamps.sol";

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

    function initialize(address initialOwner, address uts, address l1Messenger, address l2AnchoringManager)
        public
        initializer
    {
        __Ownable_init(initialOwner);

        require(uts != address(0), "UTS: Invalid UniversalTimestamps address");
        require(l1Messenger != address(0), "UTS: Invalid L1ScrollMessenger address");
        require(l2AnchoringManager != address(0), "UTS: Invalid L2AnchoringManagerL2 address");

        L1AnchoringGatewayStorage.Storage storage $ = L1AnchoringGatewayStorage.get();
        $.uts = IUniversalTimestamps(uts);
        $.l1Messenger = IL1ScrollMessenger(l1Messenger);
        $.l2AnchoringManager = IL2AnchoringManager(l2AnchoringManager);
    }

    /// @inheritdoc IL1AnchoringGateway
    function submitBatch(bytes32 merkleRoot, uint256 startIndex, uint256 count) external payable nonReentrant {
        L1AnchoringGatewayStorage.Storage storage $ = L1AnchoringGatewayStorage.get();

        require(address($.l1Messenger) != address(0), "UTS: L1 Scroll Messenger not set");
        require(address($.l2AnchoringManager) != address(0), "UTS: L2 Anchoring Manager not set");

        uint256 attestedBlockNumber;
        try $.uts.attest(merkleRoot) {
            attestedBlockNumber = block.number;
        } catch {
            attestedBlockNumber = $.uts.blockNumberOf(merkleRoot);
            require(attestedBlockNumber != 0, "UTS: Merkle root not attested on L1");
        }

        bytes memory message = abi.encodeCall(
            IL2AnchoringManager.confirmL1AnchoringBatch, (merkleRoot, startIndex, count, attestedBlockNumber)
        );

        $.l1Messenger.sendMessage{value: msg.value}(
            address($.l2AnchoringManager),
            0,
            message,
            1_000_000, // TODO: Estimate proper gas limit
            msg.sender // refund the caller for the gas cost of L2 execution
        );

        emit BatchSubmitted(merkleRoot, startIndex, count, msg.sender);
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
}
