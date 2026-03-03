// SPDX-License-Identifier: MIT

pragma solidity =0.8.28;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {ReentrancyGuardTransient} from "@openzeppelin/contracts/utils/ReentrancyGuardTransient.sol";
import {IL2AnchoringManager} from "./IL2AnchoringManager.sol";
import {L2AnchoringManagerStorage} from "./L2AnchoringManagerStorage.sol";
import {IFeeOracle} from "../oracle/IFeeOracle.sol";
import {L2AnchoringManagerTypes} from "./L2AnchoringManagerTypes.sol";
import {MerkleTree} from "../../core/MerkleTree.sol";
import {IL2ScrollMessenger} from "scroll-contracts/L2/IL2ScrollMessenger.sol";
import {ScrollConstants} from "scroll-contracts/libraries/constants/ScrollConstants.sol";
import {
    AccessControlDefaultAdminRulesUpgradeable
} from "@openzeppelin/contracts-upgradeable/access/extensions/AccessControlDefaultAdminRulesUpgradeable.sol";
import {ERC721Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC721/ERC721Upgradeable.sol";
import {EASHelper} from "../../core/EASHelper.sol";
import {Attestation, IEAS} from "eas-contracts/IEAS.sol";
import {INFTGenerator} from "../nft/INFTGenerator.sol";

contract L2AnchoringManager is
    Initializable,
    UUPSUpgradeable,
    ReentrancyGuardTransient,
    AccessControlDefaultAdminRulesUpgradeable,
    ERC721Upgradeable,
    IL2AnchoringManager
{
    bytes32 public constant FEE_COLLECTOR_ROLE = keccak256("FEE_COLLECTOR_ROLE");

    error AlreadyInitialized();
    error InvalidAddress();

    error InsufficientFee();
    error RefundFailed();

    error InvalidBatchCount();
    error InvalidL2Messenger();
    error InvalidL1Sender();
    error BatchAlreadyExists();
    error InvalidBatchOrder();

    error MerkleRootMismatch();

    error InvalidAttestationId();
    error NoPendingBatch();

    error InvalidBatchIndexHint();
    error NFTAlreadyClaimed();

    error InvalidAmount();
    error WithdrawalFailed();

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    /// @notice For deterministic deployment, we use a separate initialize function instead of the constructor.
    function initialize() public initializer {
        __AccessControlDefaultAdminRules_init(0, msg.sender);
        __ERC721_init("UTS Anchoring Certificate", "UTS");

        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        // Start from 1 to use 0 as a sentinel value
        $.queueIndex = 1;
        $.confirmedIndex = 1;

        _setRoleAdmin(FEE_COLLECTOR_ROLE, DEFAULT_ADMIN_ROLE);
    }

    /// @notice For deterministic deployment, we use a separate lateInitialize function to transfer ownership,
    /// and setup any necessary parameters that are not provided at the time of deployment.
    function lateInitialize(
        string memory l2Name,
        address newAdmin,
        address eas,
        address feeOracle,
        address l2Messenger,
        address nftGeneratorProxy
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        if ($.initialized) revert AlreadyInitialized();
        $.initialized = true;
        $.l2Name = l2Name;

        if (
            newAdmin == address(0) || eas == address(0) || feeOracle == address(0) || l2Messenger == address(0)
                || nftGeneratorProxy == address(0)
        ) {
            revert InvalidAddress();
        }

        setFeeOracle(feeOracle);
        setL2Messenger(l2Messenger);

        $.eas = IEAS(eas);
        $.nftGeneratorProxy = INFTGenerator(nftGeneratorProxy);

        beginDefaultAdminTransfer(newAdmin);
    }

    /// @notice Completes the ownership transfer process and sets a delay for future admin transfers.
    function completeInitialization() external {
        acceptDefaultAdminTransfer();
        changeDefaultAdminDelay(3 days);
    }

    /// @inheritdoc IL2AnchoringManager
    function submitForL1Anchoring(bytes32 root) external payable {
        submitForL1Anchoring(root, _msgSender());
    }

    /// @inheritdoc IL2AnchoringManager
    function submitForL1Anchoring(bytes32 root, address refundAddress) public payable nonReentrant {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();

        if (address($.eas) == address(0)) revert InvalidAddress();

        uint256 requiredFee = $.feeOracle.getFloorFee();
        if (msg.value < requiredFee) revert InsufficientFee();

        bytes32 attestationId = EASHelper.attest($.eas, root);

        uint256 currentIndex = $.queueIndex++;
        $.indexToRecords[currentIndex] = L2AnchoringManagerTypes.AnchoringRecord({
            root: root, attestationId: attestationId, blockNumber: block.number
        });
        $.rootToAttestationId[root] = attestationId;
        $.attestationIdToIndex[attestationId] = currentIndex;

        emit L1AnchoringQueued(attestationId, root, currentIndex, requiredFee, block.number, block.timestamp);

        // refund fee to `refundAddress`
        unchecked {
            uint256 _refund = msg.value - requiredFee;
            if (_refund > 0) {
                (bool _success,) = refundAddress.call{value: _refund}("");
                if (!_success) revert RefundFailed();
            }
        }
    }

    /// @inheritdoc IL2AnchoringManager
    function isConfirmed(bytes32 root) public view returns (bool) {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        uint256 index = $.attestationIdToIndex[$.rootToAttestationId[root]];
        return index != 0 && index < $.confirmedIndex;
    }

    /// @inheritdoc IL2AnchoringManager
    function notifyAnchored(
        bytes32 claimedRoot,
        uint256 startIndex,
        uint256 count,
        uint256 l1Timestamp,
        uint256 l1BlockNumber
    ) external {
        if (count == 0) revert InvalidBatchCount();

        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        if (address($.l1Gateway) == address(0)) revert InvalidAddress();

        if (msg.sender != address($.l2Messenger)) revert InvalidL2Messenger();
        address l1Sender = $.l2Messenger.xDomainMessageSender();
        if (l1Sender != $.l1Gateway) revert InvalidL1Sender();

        /// Require there's no pending batch to prevent overlapping batches which can cause state inconsistency
        if ($.pendingBatch.count != 0) revert BatchAlreadyExists();
        /// Require the batch to be in order to prevent skipping or reordering batches which can cause state inconsistency
        if (startIndex != $.confirmedIndex) revert InvalidBatchOrder();

        // Store the batch details for later finalization
        $.pendingBatch = L2AnchoringManagerTypes.PendingL1Batch({
            claimedRoot: claimedRoot,
            startIndex: startIndex,
            count: count,
            l1Timestamp: l1Timestamp,
            l1BlockNumber: l1BlockNumber
        });

        emit L1BatchArrived(claimedRoot, startIndex, count, l1Timestamp, l1BlockNumber, block.number, block.timestamp);
    }

    /// @inheritdoc IL2AnchoringManager
    function finalizeBatch() external {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        L2AnchoringManagerTypes.PendingL1Batch memory batch = $.pendingBatch;

        if (batch.count == 0) revert NoPendingBatch();

        bytes32[] memory leaves = new bytes32[](batch.count);
        for (uint256 i = 0; i < batch.count; i++) {
            uint256 index = batch.startIndex + i;
            bytes32 root = $.indexToRecords[index].root;
            leaves[i] = root;
        }

        bytes32 computedRoot = MerkleTree.computeRoot(leaves);
        if (computedRoot != batch.claimedRoot) revert MerkleRootMismatch();

        $.confirmedIndex = batch.startIndex + batch.count;

        emit L1BatchFinalized(
            batch.claimedRoot,
            batch.startIndex,
            batch.count,
            batch.l1Timestamp,
            batch.l1BlockNumber,
            block.number,
            block.timestamp
        );

        $.batches[batch.startIndex] = L2AnchoringManagerTypes.L1Batch({
            count: batch.count, l1Timestamp: batch.l1Timestamp, l1BlockNumber: batch.l1BlockNumber
        });

        // Clear the pending batch
        delete $.pendingBatch;
    }

    /// @inheritdoc IL2AnchoringManager
    // forge-lint: disable-next-line(mixed-case-function)
    function claimNFT(bytes32 attestationId, uint256 batchStartIndexHint) public nonReentrant {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();

        uint256 index = $.attestationIdToIndex[attestationId];
        if (index == 0) revert InvalidAttestationId();
        if ($.nftClaimedAndHint[index] != 0) revert NFTAlreadyClaimed();

        L2AnchoringManagerTypes.L1Batch memory batch = $.batches[batchStartIndexHint];

        // check if the batch details exist
        if (batch.count == 0) revert InvalidBatchIndexHint();
        // check in range of the batch
        if (!(index >= batchStartIndexHint && index < batchStartIndexHint + batch.count)) {
            revert InvalidBatchIndexHint();
        }

        Attestation memory request = $.eas.getAttestation(attestationId);
        bytes32 root = abi.decode(request.data, (bytes32));

        $.nftClaimedAndHint[index] = batchStartIndexHint;

        _safeMint(request.attester, index, bytes.concat(root));
        emit NFTClaimed(request.attester, index, root, block.timestamp);
    }

    // forge-lint: disable-next-line(mixed-case-function)
    function tokenURI(uint256 tokenId) public view virtual override returns (string memory) {
        _requireOwned(tokenId);
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();

        L2AnchoringManagerTypes.AnchoringRecord memory record = $.indexToRecords[tokenId];

        uint256 batchStartIndexHint = $.nftClaimedAndHint[tokenId];
        L2AnchoringManagerTypes.L1Batch memory batch = $.batches[batchStartIndexHint];
        if (batch.count == 0) revert InvalidBatchIndexHint();

        return $.nftGeneratorProxy
            .generateTokenURI(
                tokenId, record.root, record.blockNumber, batch.l1BlockNumber, batch.l1Timestamp, $.l2Name
            );
    }

    // --- Admin Functions ---

    function setFeeOracle(address oracle) public onlyRole(DEFAULT_ADMIN_ROLE) {
        if (oracle == address(0)) revert InvalidAddress();
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        address oldOracle = address($.feeOracle);
        $.feeOracle = IFeeOracle(oracle);
        emit FeeOracleUpdated(oldOracle, oracle);
    }

    function setL1Gateway(address l1Gateway) public onlyRole(DEFAULT_ADMIN_ROLE) {
        if (l1Gateway == address(0)) revert InvalidAddress();
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        address oldGateway = $.l1Gateway;
        $.l1Gateway = l1Gateway;
        emit L1GatewayUpdated(oldGateway, l1Gateway);
    }

    function setL2Messenger(address l2Messenger) public onlyRole(DEFAULT_ADMIN_ROLE) {
        if (l2Messenger == address(0)) revert InvalidAddress();
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        IL2ScrollMessenger messenger = IL2ScrollMessenger(l2Messenger);
        // Sanity check to ensure it's a valid messenger
        require(
            messenger.xDomainMessageSender() == ScrollConstants.DEFAULT_XDOMAIN_MESSAGE_SENDER,
            "UTS: Invalid L2 Messenger"
        );
        $.l2Messenger = messenger;
        emit L2MessengerUpdated(address($.l2Messenger), l2Messenger);
    }

    /// @inheritdoc IL2AnchoringManager
    function clearBatch() external onlyRole(DEFAULT_ADMIN_ROLE) {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        delete $.pendingBatch;
    }

    /// @inheritdoc IL2AnchoringManager
    function withdrawFees(address to, uint256 amount) external nonReentrant onlyRole(FEE_COLLECTOR_ROLE) {
        if (to == address(0)) revert InvalidAddress();
        if (amount == 0 || amount > address(this).balance) revert InvalidAmount();

        (bool success,) = payable(to).call{value: amount}("");
        if (!success) revert WithdrawalFailed();

        emit FeesWithdrawn(to, amount);
    }

    // --- Others ---

    function _authorizeUpgrade(address newImplementation) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}

    /// @dev See {IERC165-supportsInterface}.
    function supportsInterface(bytes4 interfaceId)
        public
        view
        virtual
        override(AccessControlDefaultAdminRulesUpgradeable, ERC721Upgradeable)
        returns (bool)
    {
        return AccessControlDefaultAdminRulesUpgradeable.supportsInterface(interfaceId)
            || ERC721Upgradeable.supportsInterface(interfaceId);
    }

    receive() external payable {}
}
