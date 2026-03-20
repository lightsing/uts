# NFT Certificates

After a batch is finalized, users can claim an ERC-721 NFT certificate that serves as a visual, on-chain proof that their content hash was anchored on L1 Ethereum.

## Claiming

Users call `claimNFT` on the `L2AnchoringManager`:

```solidity
function claimNFT(
    bytes32 attestationId,
    uint256 batchStartIndexHint
) external nonReentrant
```

### Requirements

1. The attestation must exist and be mapped to a queue index.
2. The queue index must be confirmed (`index < confirmedIndex`).
3. The NFT must not already be claimed.
4. The `batchStartIndexHint` must point to the correct batch containing this index.

### Minting

The token ID equals the queue index. The NFT is minted to the original attester (the address that created the EAS attestation), not necessarily `msg.sender`.

## On-Chain Metadata

The `tokenURI` function generates fully on-chain metadata — no IPFS or external hosting required:

```solidity
function tokenURI(uint256 tokenId) public view returns (string memory)
```

It returns a `data:application/json;base64,...` URI containing:

```json
{
  "name": "Certificate #<tokenId> - <l2Name>",
  "description": "Proof of content existence at timestamp ...",
  "external_url": "https://timestamps.now/<chainId>/<tokenId>",
  "image": "data:image/svg+xml;base64,...",
  "attributes": [
    { "display_type": "date", "trait_type": "date", "value": <unix_timestamp> },
    { "trait_type": "l1BlockNumber", "value": "<block>" },
    { "trait_type": "l2BlockNumber", "value": "<block>" }
  ]
}
```

## SVG Generation

The `NFTGenerator` contract produces a complex SVG certificate design entirely on-chain. The visual includes:

- **Gradient background** with animated glow effects.
- **Grid pattern** overlay.
- **Tree diagram** symbolizing the Merkle tree structure.
- **Certificate ID** (token ID with comma formatting).
- **Content hash** displayed as two lines of 32 hex characters.
- **L2 block number** of the original submission.
- **L1 block number** of the anchoring transaction.
- **Timestamp** formatted as `YYYY-MM-DD HH:MM:SS` (using Solady's `DateTime` library).
- **Code 128-C barcode** encoding the token ID.
- **"UNIVERSAL TIMESTAMPS PROTOCOL"** watermark.

## Code 128-C Barcode

The `Code128CGenerator` contract generates a Code 128-C barcode as an SVG element:

1. The token ID is zero-padded to 20 digits.
2. Digits are grouped into pairs (Code 128-C encodes two digits per symbol).
3. A checksum is calculated using the Code 128-C weighted sum algorithm.
4. The barcode is rendered as alternating white and blue bars in an SVG `<g>` element.

This allows the certificate to be scanned and linked back to the on-chain record.

## Design Rationale

Generating NFT metadata on-chain (rather than pointing to IPFS) ensures:

- **Permanence**: The metadata cannot disappear if an IPFS pin is removed.
- **Trustlessness**: Anyone can verify the metadata by calling the contract directly.
- **Consistency**: The visual representation always matches the on-chain state.

The SVG for the NFT is generated every time the `tokenURI` is called.
