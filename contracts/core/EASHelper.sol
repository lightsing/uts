// SPDX-License-Identifier: MIT

pragma solidity =0.8.28;

import {IEAS, AttestationRequest, AttestationRequestData} from "eas-contracts/IEAS.sol";

library EASHelper {
    /// @notice Schema ID for content hash attestation, un-revokable, with no extra data, schema: bytes32 contentHash
    bytes32 constant CONTENT_HASH_SCHEMA = 0x5c5b8b295ff43c8e442be11d569e94a4cd5476f5e23df0f71bdd408df6b9649c;

    /// @notice Helper function to create an attestation for a given content hash using the EAS contract
    function attest(IEAS eas, bytes32 root) public returns (bytes32) {
        return eas.attest(getAttestationRequest(root));
    }

    function getAttestationRequest(bytes32 root) public pure returns (AttestationRequest memory) {
        return AttestationRequest({
            schema: CONTENT_HASH_SCHEMA,
            data: AttestationRequestData({
                recipient: address(0), // No specific recipient, as the attestation is about the content hash
                expirationTime: 0, // No expiration
                revocable: false, // Un-revokable
                refUID: bytes32(0), // No reference to another attestation
                data: abi.encode(root), // Encode the root in the data field
                value: 0 // No ETH value needed for this attestation
            })
        });
    }
}
