// SPDX-License-Identifier: MIT

pragma solidity ^0.8.29;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {ReentrancyGuardTransient} from "@openzeppelin/contracts/utils/ReentrancyGuardTransient.sol";
import {L1AnchoringGatewayStorage} from "./L1AnchoringGatewayStorage.sol";
import {IL1AnchoringGateway} from "./IL1AnchoringGateway.sol";
import {IL1ScrollMessenger} from "scroll-contracts/L1/IL1ScrollMessenger.sol";
import {IL2AnchoringManager} from "../L2/manager/IL2AnchoringManager.sol";
import {IUniversalTimestamps} from "../core/IUniversalTimestamps.sol";
import {
    AccessControlDefaultAdminRulesUpgradeable
} from "@openzeppelin/contracts-upgradeable/access/extensions/AccessControlDefaultAdminRulesUpgradeable.sol";

contract L1AnchoringGateway is
    Initializable,
    UUPSUpgradeable,
    ReentrancyGuardTransient,
    AccessControlDefaultAdminRulesUpgradeable,
    IL1AnchoringGateway
{
    bytes32 public constant SUBMITTER_ROLE = keccak256("SUBMITTER_ROLE");

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize(address initialOwner, address uts, address l1Messenger, address l2AnchoringManager)
        public
        initializer
    {
        __AccessControlDefaultAdminRules_init(3 days, initialOwner);

        require(uts != address(0), "UTS: Invalid UniversalTimestamps address");
        require(l1Messenger != address(0), "UTS: Invalid L1ScrollMessenger address");
        require(l2AnchoringManager != address(0), "UTS: Invalid L2AnchoringManager address");

        L1AnchoringGatewayStorage.Storage storage $ = L1AnchoringGatewayStorage.get();
        $.uts = IUniversalTimestamps(uts);
        $.l1Messenger = IL1ScrollMessenger(l1Messenger);
        $.l2AnchoringManager = IL2AnchoringManager(l2AnchoringManager);

        // Set up roles
        grantRole(SUBMITTER_ROLE, initialOwner);
        _setRoleAdmin(SUBMITTER_ROLE, DEFAULT_ADMIN_ROLE);
    }

    /// @inheritdoc IL1AnchoringGateway
    function submitBatch(bytes32 merkleRoot, uint256 startIndex, uint256 count, uint256 gasLimit)
        external
        payable
        nonReentrant
        onlyRole(SUBMITTER_ROLE)
    {
        L1AnchoringGatewayStorage.Storage storage $ = L1AnchoringGatewayStorage.get();

        require(address($.l1Messenger) != address(0), "UTS: L1 Scroll Messenger not set");
        require(address($.l2AnchoringManager) != address(0), "UTS: L2 Anchoring Manager not set");

        uint256 attestedBlockNumber;
        try $.uts.attest(merkleRoot) {
            attestedBlockNumber = block.number;
        } catch {
            (attestedBlockNumber,) = $.uts.blockNumberOf(merkleRoot);
            require(attestedBlockNumber != 0, "UTS: Merkle root not attested on L1");
        }

        bytes memory message =
            abi.encodeCall(IL2AnchoringManager.notifyAnchored, (merkleRoot, startIndex, count, attestedBlockNumber));

        $.l1Messenger.sendMessage{value: msg.value}(
            address($.l2AnchoringManager),
            0,
            message,
            gasLimit,
            _msgSender() // refund the caller for the gas cost of L2 execution
        );

        emit BatchSubmitted(merkleRoot, startIndex, count, _msgSender());
    }

    // -- Admin functions --
    function setUts(address newUts) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newUts != address(0), "UTS: Invalid UniversalTimestamps address");
        L1AnchoringGatewayStorage.Storage storage $ = L1AnchoringGatewayStorage.get();
        address oldUts = address($.uts);
        $.uts = IUniversalTimestamps(newUts);
        emit UTSUpdated(oldUts, newUts);
    }

    function setL1ScrollMessenger(address newMessenger) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newMessenger != address(0), "UTS: Invalid L1 Scroll Messenger address");
        L1AnchoringGatewayStorage.Storage storage $ = L1AnchoringGatewayStorage.get();
        address oldMessenger = address($.l1Messenger);
        $.l1Messenger = IL1ScrollMessenger(newMessenger);
        emit L1ScrollMessengerUpdated(oldMessenger, newMessenger);
    }

    function setL2AnchoringManager(address newManager) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newManager != address(0), "UTS: Invalid L2 Anchoring Manager address");
        L1AnchoringGatewayStorage.Storage storage $ = L1AnchoringGatewayStorage.get();
        address oldManager = address($.l2AnchoringManager);
        $.l2AnchoringManager = IL2AnchoringManager(newManager);
        emit L2AnchoringManagerUpdated(oldManager, newManager);
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}
}
