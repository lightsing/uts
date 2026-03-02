//! Solidity contracts for UTS

/// EAS contract
pub mod eas {
    use alloy_primitives::{Address, address};
    use alloy_sol_types::sol;

    sol! {
        /// @notice A struct representing the arguments of the attestation request.
        struct AttestationRequestData {
            address recipient; // The recipient of the attestation.
            uint64 expirationTime; // The time when the attestation expires (Unix timestamp).
            bool revocable; // Whether the attestation is revocable.
            bytes32 refUID; // The UID of the related attestation.
            bytes data; // Custom attestation data.
            uint256 value; // An explicit ETH amount to send to the resolver. This is important to prevent accidental user errors.
        }

        /// @notice A struct representing the full arguments of the attestation request.
        struct AttestationRequest {
            bytes32 schema; // The unique identifier of the schema.
            AttestationRequestData data; // The arguments of the attestation request.
        }

        interface IEAS {
            /// @notice Emitted when an attestation has been made.
            /// @param recipient The recipient of the attestation.
            /// @param attester The attesting account.
            /// @param uid The UID of the new attestation.
            /// @param schemaUID The UID of the schema.
            event Attested(address indexed recipient, address indexed attester, bytes32 uid, bytes32 indexed schemaUID);

        }
    }
}

/// ERC-1967 Proxy contract
#[cfg(any(test, feature = "erc1967"))]
pub mod erc1967 {
    mod binding {
        use alloy_sol_types::sol;

        sol!(
            #[sol(rpc)]
            ERC1967Proxy,
            "abi/ERC1967Proxy.json"
        );
    }

    pub use binding::ERC1967Proxy::*;
}
