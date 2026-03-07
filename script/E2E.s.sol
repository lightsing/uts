// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {IL2AnchoringManager} from "../contracts/L2/manager/IL2AnchoringManager.sol";
import {IL1AnchoringGateway} from "../contracts/L1/IL1AnchoringGateway.sol";
import {IFeeOracle} from "../contracts/L2/oracle/IFeeOracle.sol";
import {MerkleTree} from "../contracts/core/MerkleTree.sol";
import {IEAS, AttestationRequest, AttestationRequestData} from "eas-contracts/IEAS.sol";
import {EASHelper} from "../contracts/core/EASHelper.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";

contract SubmitEAS is Script {
    function run() public {
        address _eas = vm.envAddress("EAS_SCROLL");
        IEAS eas = IEAS(_eas);

        vm.startBroadcast();
        eas.attest(
            AttestationRequest({
                schema: EASHelper.CONTENT_HASH_SCHEMA,
                data: AttestationRequestData({
                    recipient: address(0), // No specific recipient, as the attestation is about the content hash
                    expirationTime: 0, // No expiration
                    revocable: false, // Un-revokable
                    refUID: bytes32(0), // No reference to another attestation
                    data: abi.encode(0x0000000000000000000000000000000000000000000000000000000000000007), // Encode the root in the data field
                    value: 0 // No ETH value needed for this attestation
                })
            })
        );
        vm.stopBroadcast();
    }
}

contract SubmitAnchoring is Script {
    IL2AnchoringManager manager;
    IFeeOracle oracle;

    function run() public {
        address anchoringManager = vm.envAddress("ANCHORING_MANAGER");
        address feeOracle = vm.envAddress("FEE_ORACLE");

        manager = IL2AnchoringManager(anchoringManager);
        oracle = IFeeOracle(feeOracle);

        vm.startBroadcast();
        // submit(0x45c9a61d29a852778fc2d3fd4f955136dd2a49d633d6431bbe379842102518d7);
        // submit(0xcbca13c4d782ed3ee2306f4c0aa21112cea47e920f10ccac48ce1fd135917387);
        submit(0x4168A6777EC5730C480A613661ED418790BF31A6B47ABC6602072DD2E74D6F36);
        // submit(0x7cbb36e4a87098984ff38a1f07c8d493eaed57992d116322b7bc2354e1b9a608);
        // submit(0xE47C7963CBD803028A83A00B33C27DC5ABF4790400385F3EE5268BA645C2F401);
        vm.stopBroadcast();
    }

    function submit(bytes32 attestationId) internal virtual {
        uint256 fee = oracle.getFloorFee() * 1.1e18 / 1e18;
        console.log("Current floor fee:", fee);

        manager.submitForL1Anchoring{value: fee}(attestationId);
    }
}

contract SubmitBatch is Script {
    function run() public {
        address anchoringGateway = vm.envAddress("ANCHORING_GATEWAY");

        IL1AnchoringGateway gateway = IL1AnchoringGateway(anchoringGateway);

        bytes32[] memory roots = new bytes32[](5);
        roots[0] = 0x0000000000000000000000000000000000000000000000000000000000000002;
        roots[1] = 0x0000000000000000000000000000000000000000000000000000000000000003;
        roots[2] = 0x0000000000000000000000000000000000000000000000000000000000000001;
        roots[3] = 0x0000000000000000000000000000000000000000000000000000000000000000;
        roots[4] = 0x0000000000000000000000000000000000000000000000000000000000000004;

        bytes32 root = MerkleTree.computeRoot(roots);

        vm.startBroadcast();
        gateway.submitBatch{value: 0.1 ether}(root, 1, 5, 200_000);
        vm.stopBroadcast();
    }
}

contract FinalizeBatch is Script {
    function run() public {
        address anchoringManager = vm.envAddress("ANCHORING_MANAGER");

        IL2AnchoringManager manager = IL2AnchoringManager(anchoringManager);

        vm.startBroadcast();
        manager.finalizeBatch();
        vm.stopBroadcast();
    }
}

contract MintNFT is Script {
    function run() public {
        address anchoringManager = vm.envAddress("ANCHORING_MANAGER");

        IL2AnchoringManager manager = IL2AnchoringManager(anchoringManager);

        vm.startBroadcast();
        manager.claimNFT(0x45c9a61d29a852778fc2d3fd4f955136dd2a49d633d6431bbe379842102518d7, 1);
        manager.claimNFT(0xcbca13c4d782ed3ee2306f4c0aa21112cea47e920f10ccac48ce1fd135917387, 1);
        manager.claimNFT(0xb2135365d636d3f9674c5425fdee7f0e3a1b09e2e37814100e7cded8d6d63301, 1);
        manager.claimNFT(0x7cbb36e4a87098984ff38a1f07c8d493eaed57992d116322b7bc2354e1b9a608, 1);
        manager.claimNFT(0xb3c7afbdcf6213aaf8aef4367a74199d6aa97a7c0aea5c302dd279e2a667c54d, 1);
        vm.stopBroadcast();
    }
}
