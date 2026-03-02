// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {EAS} from "eas-contracts/EAS.sol";
import {IEAS} from "eas-contracts/IEAS.sol";
import {ISchemaRegistry} from "eas-contracts/ISchemaRegistry.sol";
import {SchemaRegistry} from "eas-contracts/SchemaRegistry.sol";
import {ISchemaResolver} from "eas-contracts/resolver/ISchemaResolver.sol";
import {EASHelper} from "../contracts/core/EASHelper.sol";

contract TestEASHelper {
    IEAS public eas;
    ISchemaRegistry public schemaRegistry;

    constructor() {
        schemaRegistry = new SchemaRegistry();
        bytes32 schemaId = schemaRegistry.register("bytes32 contentHash", ISchemaResolver(address(0)), false);
        require(schemaId == EASHelper.CONTENT_HASH_SCHEMA, "Schema ID mismatch");
        eas = new EAS(schemaRegistry);
    }
}

contract TestEAS is Test {
    IEAS eas;

    function setUp() public {
        eas = new TestEASHelper().eas();
    }

    function test_AttestGas() public {
        uint256 initialGas = gasleft();
        EASHelper.attest(eas, bytes32(0));
        uint256 gasUsed = initialGas - gasleft();
        console.log("Gas used for attestation:", gasUsed);
    }
}
