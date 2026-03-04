// SPDX-License-Identifier: MIT

pragma solidity =0.8.28;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {ReentrancyGuardTransient} from "@openzeppelin/contracts/utils/ReentrancyGuardTransient.sol";
import {L1AnchoringGatewayStorage} from "./L1AnchoringGatewayStorage.sol";
import {IL1AnchoringGateway} from "./IL1AnchoringGateway.sol";
import {IL1ScrollMessenger} from "scroll-contracts/L1/IL1ScrollMessenger.sol";
import {IL2AnchoringManager} from "../L2/manager/IL2AnchoringManager.sol";
import {
    AccessControlDefaultAdminRulesUpgradeable
} from "@openzeppelin/contracts-upgradeable/access/extensions/AccessControlDefaultAdminRulesUpgradeable.sol";
import {IEAS} from "eas-contracts/IEAS.sol";

contract L1AnchoringGateway is
    Initializable,
    UUPSUpgradeable,
    ReentrancyGuardTransient,
    AccessControlDefaultAdminRulesUpgradeable,
    IL1AnchoringGateway
{
    bytes32 public constant SUBMITTER_ROLE = keccak256("SUBMITTER_ROLE");

    uint256 public constant MAX_BATCH_SIZE = 512;

    /// @notice Safe bounds for gas limit of the L2 transaction to notify the L2 manager of a new batch submission.
    /// Avoid accidentally setting a gas limit that is too low causing failed transactions on the L2, or too high costly L1 fees.
    uint256 public constant MIN_GAS_LIMIT = 110_000;
    uint256 public constant MAX_GAS_LIMIT = 200_000;

    error InvalidBatchSize();
    error InvalidGasLimit();
    error InvalidAddress();

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    /// @notice For deterministic deployment, we use a separate initialize function instead of the constructor.
    function initialize(address owner) public initializer {
        __AccessControlDefaultAdminRules_init(0, owner);
        _setRoleAdmin(SUBMITTER_ROLE, DEFAULT_ADMIN_ROLE);
    }

    /// @notice For deterministic deployment, we use a separate lateInitialize function to transfer ownership,
    /// and setup any necessary parameters that are not provided at the time of deployment.
    function lateInitialize(address newAdmin, address eas, address l1Messenger, address l2AnchoringManager)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        if (newAdmin == address(0)) revert InvalidAddress();
        if (eas == address(0)) revert InvalidAddress();
        if (l1Messenger == address(0)) revert InvalidAddress();
        if (l2AnchoringManager == address(0)) revert InvalidAddress();

        L1AnchoringGatewayStorage.Storage storage $ = L1AnchoringGatewayStorage.get();
        $.eas = IEAS(eas);
        $.eas.getTimestamp(bytes32(0)); // sanity check that the EAS contract is correct

        setL1ScrollMessenger(l1Messenger);
        setL2AnchoringManager(l2AnchoringManager);

        beginDefaultAdminTransfer(newAdmin);
    }

    /// @notice Completes the ownership transfer process and sets a delay for future admin transfers.
    function completeInitialization() external {
        acceptDefaultAdminTransfer();
        changeDefaultAdminDelay(3 days);
    }

    /// @inheritdoc IL1AnchoringGateway
    function submitBatch(bytes32 merkleRoot, uint256 startIndex, uint256 count, uint256 gasLimit)
        external
        payable
        nonReentrant
        onlyRole(SUBMITTER_ROLE)
    {
        L1AnchoringGatewayStorage.Storage storage $ = L1AnchoringGatewayStorage.get();

        if (address($.l1Messenger) == address(0)) revert InvalidAddress();
        if (address($.l2AnchoringManager) == address(0)) revert InvalidAddress();

        if (count == 0 || count > MAX_BATCH_SIZE) revert InvalidBatchSize();
        if (gasLimit < MIN_GAS_LIMIT || gasLimit > MAX_GAS_LIMIT) revert InvalidGasLimit();

        uint256 blockNumber = 0;
        uint256 timestamp = $.eas.getTimestamp(merkleRoot);
        if (timestamp == 0) {
            timestamp = $.eas.timestamp(merkleRoot);
            blockNumber = block.number;
        }

        bytes memory message =
            abi.encodeCall(IL2AnchoringManager.notifyAnchored, (merkleRoot, startIndex, count, timestamp, blockNumber));

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

    function setL1ScrollMessenger(address newMessenger) public onlyRole(DEFAULT_ADMIN_ROLE) {
        if (newMessenger == address(0)) revert InvalidAddress();
        // sanity check that the new messenger is correct by calling a public view function
        //     address public immutable rollup;
        (bool success,) = newMessenger.staticcall(abi.encodeWithSignature("rollup()"));
        if (!success) revert InvalidAddress();

        L1AnchoringGatewayStorage.Storage storage $ = L1AnchoringGatewayStorage.get();
        address oldMessenger = address($.l1Messenger);
        $.l1Messenger = IL1ScrollMessenger(newMessenger);
        emit L1ScrollMessengerUpdated(oldMessenger, newMessenger);
    }

    function setL2AnchoringManager(address newManager) public onlyRole(DEFAULT_ADMIN_ROLE) {
        if (newManager == address(0)) revert InvalidAddress();
        L1AnchoringGatewayStorage.Storage storage $ = L1AnchoringGatewayStorage.get();
        address oldManager = address($.l2AnchoringManager);
        $.l2AnchoringManager = IL2AnchoringManager(newManager);
        emit L2AnchoringManagerUpdated(oldManager, newManager);
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}
}
