// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import {IL1ScrollMessenger} from "scroll-contracts/L1/IL1ScrollMessenger.sol";
import {IL2AnchoringManager} from "../L2/manager/IL2AnchoringManager.sol";
import {IUniversalTimestamps} from "../core/IUniversalTimestamps.sol";

/**
 * @dev Library containing the ERC-7201 namespace constant.
 * This keeps the implementation detail hidden from the interface.
 */
library L1AnchoringGatewayStorage {
    string internal constant NAMESPACE = "uts.storage.L1AnchoringGateway";

    /// @dev keccak256(abi.encode(uint256(keccak256("uts.storage.L1AnchoringGateway")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 internal constant SLOT = 0x8edb9fe689fd9379dceae5cf4dde34cad983b6db894e69fe7b25cb8e53843500;

    /// @custom:storage-location erc7201:uts.storage.L1AnchoringGateway
    struct Storage {
        /// @notice Reference to the Universal Timestamps contract for recording anchoring events
        IUniversalTimestamps uts;
        /// @notice Reference to the L1 Scroll Messenger contract
        IL1ScrollMessenger l1Messenger;
        /// @notice Reference to the L2 Anchoring Manager contract
        IL2AnchoringManager l2AnchoringManager;
    }

    function get() internal pure returns (L1AnchoringGatewayStorage.Storage storage $) {
        assembly ("memory-safe") {
            $.slot := SLOT
        }
    }
}
