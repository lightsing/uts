// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

import {IFeeOracle} from "../oracle/IFeeOracle.sol";
import {L2AnchoringManagerTypes} from "./L2AnchoringManagerTypes.sol";
import {IL2ScrollMessenger} from "scroll-contracts/L2/IL2ScrollMessenger.sol";
import {IEAS} from "eas-contracts/IEAS.sol";

/**
 * @dev Library containing the ERC-7201 namespace constant.
 * This keeps the implementation detail hidden from the interface.
 */
library L2AnchoringManagerStorage {
    string internal constant NAMESPACE = "uts.storage.L2AnchoringManager";

    /// @dev keccak256(abi.encode(uint256(keccak256("uts.storage.L2AnchoringManager")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 internal constant SLOT = 0x5accfd2b2bcf275f7d10bb4569421f50f846511017720654fefc7e6d91daf100;

    /// @custom:storage-location erc7201:uts.storage.L2AnchoringManager
    struct Storage {
        IEAS eas;
        IFeeOracle feeOracle;
        /// @notice Executor for L1 -> L2 messages
        IL2ScrollMessenger l2Messenger;
        /// @notice L1 sender address
        address l1Gateway;
        /// @notice Queue index for the next anchoring item to be added
        uint256 queueIndex;

        /// @notice Storage for pending L1 batch confirmation
        L2AnchoringManagerTypes.L1Batch pendingBatch;
        /// @notice Next index of the anchoring item to be confirmed
        uint256 confirmedIndex;
        /// @notice Mapping to track the L1 block number for each batch start index
        mapping(uint256 => uint256) batchStartToL1Block;

        // Mapping to track attestation id
        mapping(bytes32 => bytes32) rootToAttestationId;
        mapping(uint256 => bytes32) indexToAttestationId;
        mapping(bytes32 => uint256) attestationIdToIndex;

        string baseTokenURI;
        mapping(uint256 => bool) nftClaimed;
    }

    function get() internal pure returns (L2AnchoringManagerStorage.Storage storage $) {
        assembly ("memory-safe") {
            $.slot := SLOT
        }
    }
}
