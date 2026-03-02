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

contract L2AnchoringManager is
    Initializable,
    UUPSUpgradeable,
    ReentrancyGuardTransient,
    AccessControlDefaultAdminRulesUpgradeable,
    ERC721Upgradeable,
    IL2AnchoringManager
{
    bytes32 public constant FEE_COLLECTOR_ROLE = keccak256("FEE_COLLECTOR_ROLE");

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
        address newAdmin,
        address eas,
        address feeOracle,
        address l2Messenger,
        string memory baseTokenURI
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newAdmin != address(0), "UTS: Invalid admin address");
        require(feeOracle != address(0), "UTS: Invalid FeeOracle address");
        require(l2Messenger != address(0), "UTS: Invalid L2 Messenger address");

        setFeeOracle(feeOracle);
        setL2Messenger(l2Messenger);

        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        $.eas = IEAS(eas);
        $.baseTokenURI = baseTokenURI;

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

        require(address($.eas) != address(0), "UTS: Oracle not set");

        uint256 requiredFee = $.feeOracle.getFloorFee();
        require(msg.value >= requiredFee, "UTS: Insufficient fee for L1 anchoring");

        bytes32 attestationId = EASHelper.attest($.eas, root);

        uint256 currentIndex = $.queueIndex++;
        $.indexToRoot[currentIndex] = root;
        $.rootToAttestationId[root] = attestationId;
        $.indexToAttestationId[currentIndex] = attestationId;
        $.attestationIdToIndex[attestationId] = currentIndex;

        emit L1AnchoringQueued(attestationId, root, currentIndex, requiredFee, block.number, block.timestamp);

        // refund fee to `refundAddress`
        unchecked {
            uint256 _refund = msg.value - requiredFee;
            if (_refund > 0) {
                (bool _success,) = refundAddress.call{value: _refund}("");
                require(_success, "Failed to refund the fee");
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
        require(count > 0, "UTS: Count must be greater than zero");

        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        require(address($.l1Gateway) != address(0), "UTS: L1 Gateway not set");

        require(msg.sender == address($.l2Messenger), "UTS: Unauthorized caller");
        address l1Sender = $.l2Messenger.xDomainMessageSender();
        require(l1Sender == $.l1Gateway, "UTS: Invalid L1 sender");

        /// Require there's no pending batch to prevent overlapping batches which can cause state inconsistency
        require($.pendingBatch.count == 0, "UTS: Pending batch already exists");

        // Store the batch details for later finalization
        $.pendingBatch = L2AnchoringManagerTypes.L1Batch({
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
        L2AnchoringManagerTypes.L1Batch memory batch = $.pendingBatch;

        require(batch.count > 0, "UTS: No pending batch");

        bytes32[] memory leaves = new bytes32[](batch.count);
        for (uint256 i = 0; i < batch.count; i++) {
            uint256 index = batch.startIndex + i;
            bytes32 root = $.indexToRoot[index];
            leaves[i] = root;
        }

        bytes32 computedRoot = MerkleTree.computeRoot(leaves);
        require(computedRoot == batch.claimedRoot, "UTS: Invalid Merkle Root");

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

        // Clear the pending batch
        delete $.pendingBatch;
    }

    /// @inheritdoc IL2AnchoringManager
    // forge-lint: disable-next-line(mixed-case-function)
    function claimNFT(bytes32 attestationId) public nonReentrant {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();

        uint256 index = $.attestationIdToIndex[attestationId];
        require(index != 0 && index < $.confirmedIndex, "UTS: Invalid or unconfirmed index");
        require(!$.nftClaimed[index], "UTS: NFT already claimed");

        Attestation memory request = $.eas.getAttestation(attestationId);
        bytes32 root = abi.decode(request.data, (bytes32));

        $.nftClaimed[index] = true;

        _safeMint(request.attester, index, bytes.concat(root));
        emit NFTClaimed(request.attester, index, root, block.timestamp);
    }

    /// @inheritdoc IL2AnchoringManager
    function getBaseURI() external view returns (string memory) {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        return $.baseTokenURI;
    }

    // --- Admin Functions ---

    function setFeeOracle(address _oracle) public onlyRole(DEFAULT_ADMIN_ROLE) {
        require(address(_oracle) != address(0), "UTS: Invalid Oracle");
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        address oldOracle = address($.feeOracle);
        $.feeOracle = IFeeOracle(_oracle);
        emit FeeOracleUpdated(oldOracle, _oracle);
    }

    function setL1Gateway(address l1Gateway) public onlyRole(DEFAULT_ADMIN_ROLE) {
        require(l1Gateway != address(0), "UTS: Invalid L1 Gateway address");
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        address oldGateway = $.l1Gateway;
        $.l1Gateway = l1Gateway;
        emit L1GatewayUpdated(oldGateway, l1Gateway);
    }

    function setL2Messenger(address l2Messenger) public onlyRole(DEFAULT_ADMIN_ROLE) {
        require(l2Messenger != address(0), "UTS: Invalid L2 Messenger address");
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

    function setURI(string memory baseTokenURI) public onlyRole(DEFAULT_ADMIN_ROLE) {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        $.baseTokenURI = baseTokenURI;
        emit BaseURIUpdated($.baseTokenURI, baseTokenURI);
    }

    /// @inheritdoc IL2AnchoringManager
    function clearBatch() external onlyRole(DEFAULT_ADMIN_ROLE) {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        delete $.pendingBatch;
    }

    /// @inheritdoc IL2AnchoringManager
    function withdrawFees(address to, uint256 amount) external nonReentrant onlyRole(FEE_COLLECTOR_ROLE) {
        require(to != address(0), "UTS: Invalid address");
        require(amount > 0 && amount <= address(this).balance, "UTS: Invalid amount");

        (bool success,) = payable(to).call{value: amount}("");
        require(success, "UTS: Withdrawal failed");

        emit FeesWithdrawn(to, amount);
    }

    // --- Others ---

    // forge-lint: disable-next-line(mixed-case-function)
    function _baseURI() internal view virtual override returns (string memory) {
        L2AnchoringManagerStorage.Storage storage $ = L2AnchoringManagerStorage.get();
        return $.baseTokenURI;
    }

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
