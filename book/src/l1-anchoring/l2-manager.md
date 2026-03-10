# L2AnchoringManager

The `L2AnchoringManager` is the L2-side orchestrator for the L1 anchoring pipeline. It manages a queue of user-submitted attestation roots, receives cross-chain notifications from L1, verifies batch integrity, and mints NFT certificates.

## submitForL1Anchoring

Users call this function to request L1 anchoring for an existing EAS attestation:

```solidity
function submitForL1Anchoring(
    bytes32 attestationId
) external payable nonReentrant
```

### Validation Steps

1. **Duplicate check**: The attestation must not already be submitted.
2. **Fee check**: `msg.value >= feeOracle.getFloorFee()`.
3. **Attestation validation**:
   - Schema must be `CONTENT_HASH_SCHEMA`.
   - Expiration must be `0` (non-expiring).
   - Must be non-revocable.
4. **Decode root**: `abi.decode(attestation.data, (bytes32))`.

### Storage

```solidity
struct AnchoringRecord {
    bytes32 root;            // User's content hash
    bytes32 attestationId;   // EAS attestation ID
    uint256 blockNumber;     // L2 block of submission
}

indexToRecords[queueIndex] = record;
attestationIdToIndex[attestationId] = queueIndex;
queueIndex++;
```

The `queueIndex` starts at 1 and increments monotonically. Index 0 is reserved as a sentinel for "not found".

### Fee Refund

If the user overpays, excess ETH is refunded to a configurable refund address (defaults to `msg.sender`).

## FeeOracle

The `FeeOracle` calculates the per-item fee for L1 anchoring based on current gas prices:

\\[ \text{fee} = \text{estimatedCost} \times \text{feeMultiplier} \div \text{expectedBatchSize} \times \text{PRECISION} \\]

Where the estimated batch cost is:

\\[ \text{estimatedCost} = \text{l1BaseFee} \times \text{l1Gas} + \text{crossDomainGasPrice} \times \text{crossDomainGas} + \text{l2BaseFee} \times \text{l2ExecutionGas} \\]

And L2 execution gas scales with batch size:

\\[ \text{l2ExecutionGas} = \text{l2ExecutionScalar} \times \text{batchSize} + \text{l2ExecutionOverhead} \\]

### Default Parameters

| Parameter                 | Default    | Description               |
| ------------------------- | ---------- | ------------------------- |
| `l1GasEstimated`          | 350,000    | Gas to attest batch on L1 |
| `crossDomainGasEstimated` | 110,000    | Gas for L1→L2 message     |
| `l2ExecutionScalar`       | 3,500      | Per-item L2 gas           |
| `l2ExecutionOverhead`     | 35,000     | Base L2 gas               |
| `expectedBatchSize`       | 256        | Assumed items per batch   |
| `feeMultiplier`           | 1.5 × 10¹⁸ | Safety margin (1.5×)      |

The fee oracle reads `l1BaseFee` from Scroll's L1 gas price oracle predeployed at `0x5300000000000000000000000000000000000002`.

## Queue Index Tracking

The manager maintains two indices:

- `queueIndex`: next available slot for new submissions (monotonically increasing).
- `confirmedIndex`: the boundary of confirmed batches. All entries with index < `confirmedIndex` are confirmed.

```
  ┌──────────┬───────────────────┬──────────────┐
  │Confirmed │    Pending Batch  │  Unprocessed │
  │ [1, ci)  │   [ci, ci+count)  │ [ci+count, qi)│
  └──────────┴───────────────────┴──────────────┘
  1         ci                              qi
```

## notifyAnchored

Called by the L2 Scroll Messenger when the L1 gateway successfully timestamps a batch:

```solidity
function notifyAnchored(
    bytes32 claimedRoot,
    uint256 startIndex,
    uint256 count,
    uint256 l1Timestamp,
    uint256 l1BlockNumber
) external
```

Guards:

- `msg.sender` must be the L2 Scroll Messenger.
- `xDomainMessageSender` must be the L1 Gateway.
- `startIndex` must equal `confirmedIndex` (sequential batches only).
- No pending batch can exist (prevents overlapping batches).

The function stores a `PendingL1Batch` for later verification and finalization.

## finalizeBatch

Anyone can call this function to complete a pending batch:

```solidity
function finalizeBatch() external nonReentrant
```

1. Load the pending batch.
2. Reconstruct the Merkle tree from stored `AnchoringRecord` roots.
3. Verify: `MerkleTree.computeRoot(leaves) == pendingBatch.claimedRoot`.
4. Update `confirmedIndex = startIndex + count`.
5. Store the finalized `L1Batch` record.
6. Clear the pending batch.

The on-chain Merkle verification ensures the relayer cannot claim a fraudulent root. The contract independently reconstructs the tree from its own stored data and compares.
