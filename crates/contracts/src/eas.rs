use alloy_primitives::{Address, B256, U256, address, b256};
use alloy_sol_types::SolValue;

/// The schema ID for the un-revocable the content hash attestation.
///
/// raw schema: `(bytes32 contentHash)`
pub const SCHEMA_ID: B256 =
    b256!("0x5c5b8b295ff43c8e442be11d569e94a4cd5476f5e23df0f71bdd408df6b9649c");

mod inner {
    alloy_sol_types::sol! {
        #![sol(all_derives)]

        /// A struct representing a single attestation.
        struct Attestation {
            bytes32 uid; // A unique identifier of the attestation.
            bytes32 schema; // The unique identifier of the schema.
            uint64 time; // The time when the attestation was created (Unix timestamp).
            uint64 expirationTime; // The time when the attestation expires (Unix timestamp).
            uint64 revocationTime; // The time when the attestation was revoked (Unix timestamp).
            bytes32 refUID; // The UID of the related attestation.
            address recipient; // The recipient of the attestation.
            address attester; // The attester/sender of the attestation.
            bool revocable; // Whether the attestation is revocable.
            bytes data; // Custom attestation data.
        }

        /// A struct representing the arguments of the attestation request.
        struct AttestationRequestData {
            address recipient; // The recipient of the attestation.
            uint64 expirationTime; // The time when the attestation expires (Unix timestamp).
            bool revocable; // Whether the attestation is revocable.
            bytes32 refUID; // The UID of the related attestation.
            bytes data; // Custom attestation data.
            uint256 value; // An explicit ETH amount to send to the resolver. This is important to prevent accidental user errors.
        }

        /// A struct representing the full arguments of the attestation request.
        struct AttestationRequest {
            bytes32 schema; // The unique identifier of the schema.
            AttestationRequestData data; // The arguments of the attestation request.
        }

        /// A struct representing the full arguments of the multi attestation request.
        struct MultiAttestationRequest {
            bytes32 schema; // The unique identifier of the schema.
            AttestationRequestData[] data; // The arguments of the attestation request.
        }

        #[sol(rpc)]
        interface IEAS {
            /// Emitted when an attestation has been made.
            ///
            /// - `recipient` The recipient of the attestation.
            /// - `attester` The attesting account.
            /// - `uid` The UID of the new attestation.
            /// - `schemaUID` The UID of the schema.
            ///
            event Attested(address indexed recipient, address indexed attester, bytes32 uid, bytes32 indexed schemaUID);

            /// Emitted when a data has been timestamped.
            ///
            /// - `data` The data.
            /// - `timestamp` The timestamp.
            ///
            event Timestamped(bytes32 indexed data, uint64 indexed timestamp);

            /// Attests to a specific schema.
            ///
            /// # Arguments
            /// The arguments of the attestation request.
            ///
            /// # Returns
            /// The UID of the new attestation.
            ///
            function attest(AttestationRequest calldata request) external payable returns (bytes32);

            /// Attests to multiple schemas.
            ///
            /// # Arguments
            /// The arguments of the multi attestation requests. The requests should be grouped by
            /// distinct schema ids to benefit from the best batching optimization.
            ///
            /// # Returns
            /// The UIDs of the new attestations.
            ///
            function multiAttest(MultiAttestationRequest[] calldata multiRequests) external payable returns (bytes32[] memory);

            /// Timestamps the specified bytes32 data.
            ///
            /// # Arguments
            /// The data to timestamp.
            ///
            /// # Returns
            /// The timestamp the data was timestamped with.
            ///
            function timestamp(bytes32 data) external returns (uint64);

            /// Returns an existing attestation by UID.
            ///
            /// # Arguments
            /// The UID of the attestation to retrieve.
            ///
            /// # Returns
            /// The attestation data members.
            ///
            function getAttestation(bytes32 uid) external view returns (Attestation memory);

            /// Returns the timestamp that the specified data was timestamped with.
            ///
            /// # Arguments
            /// The data to query.
            ///
            /// # Returns
            /// The timestamp the data was timestamped with.
            ///
            function getTimestamp(bytes32 data) external view returns (uint64);
        }
    }
    pub use IEAS::*;
}

pub use inner::{Attestation, AttestationRequest, AttestationRequestData};

pub use inner::IEASInstance as EAS;

/// events
pub mod events {
    pub use super::inner::{Attested, Timestamped};
}

/// calls
pub mod calls {
    pub use super::inner::IEAS::{
        attestCall as AttestCall, getAttestationCall as GetAttestationCall,
        getTimestampCall as GetTimestampCall, multiAttestCall as MultiAttestCall,
        timestampCall as TimestampCall,
    };
}

impl AttestationRequest {
    /// Creates a new attestation request.
    #[inline]
    pub fn new(root: B256) -> Self {
        Self {
            schema: SCHEMA_ID,
            data: AttestationRequestData {
                recipient: Address::ZERO,
                expirationTime: 0,
                revocable: false,
                refUID: B256::ZERO,
                data: root.abi_encode().into(),
                value: U256::ZERO,
            },
        }
    }
}

/// The EAS contract addresses on different chains.
pub static EAS_ADDRESSES: phf::Map<u64, Address> = phf::phf_map! {
    1u64 => address!("0xA1207F3BBa224E2c9c3c6D5aF63D0eb1582Ce587"),
    534351u64 => address!("0xaEF4103A04090071165F78D45D83A0C0782c2B2a"),
    534352u64 => address!("0xC47300428b6AD2c7D03BB76D05A176058b47E6B0"),
    11155111u64 => address!("0xC47300428b6AD2c7D03BB76D05A176058b47E6B0"),
};
