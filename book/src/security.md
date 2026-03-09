# Security Considerations

This chapter covers the security properties and protections built into the UTS protocol across both the smart contract layer and the off-chain infrastructure.

## Access Control

### Smart Contract Roles

| Role | Contract | Privilege |
|------|----------|-----------|
| `DEFAULT_ADMIN_ROLE` | L1Gateway, L2Manager, FeeOracle | Configure contract parameters, grant/revoke roles |
| `SUBMITTER_ROLE` | L1AnchoringGateway | Submit batches to L1 |
| `FEE_COLLECTOR_ROLE` | L2AnchoringManager | Withdraw accumulated fees |
| `UPDATER_ROLE` | FeeOracle | Update fee parameters |

### Admin Transfer Delay

Both `L1AnchoringGateway` and `L2AnchoringManager` use OpenZeppelin's `AccessControlDefaultAdminRulesUpgradeable` with a **3-day transfer delay** for the admin role. This prevents:

- Instant admin takeover via compromised keys.
- Flash-loan governance attacks.
- Accidental admin transfers.

The `FeeOracle` uses the non-upgradeable variant with the same 3-day delay.

## Reentrancy Protection

All state-modifying external functions use `ReentrancyGuardTransient`:

| Contract | Protected Functions |
|----------|-------------------|
| L1AnchoringGateway | `submitBatch()` |
| L2AnchoringManager | `submitForL1Anchoring()`, `claimNFT()`, `withdrawFees()` |

The transient variant (EIP-1153) uses transient storage for the reentrancy flag, saving gas compared to the traditional storage-based guard.

## Cross-Chain Message Authentication

The L2AnchoringManager validates cross-chain messages with a two-layer check:

```solidity
// In notifyAnchored():
require(msg.sender == address(l2Messenger));
require(l2Messenger.xDomainMessageSender() == l1Gateway);
```

1. **msg.sender must be the L2 Scroll Messenger** — prevents direct calls from arbitrary addresses.
2. **xDomainMessageSender must be the L1 Gateway** — prevents spoofed messages from other L1 contracts.

Both conditions must hold simultaneously. This ensures that `notifyAnchored` can only be triggered by a legitimate cross-chain message originating from the authorized L1 gateway contract.

## Merkle Proof Verification

The L2AnchoringManager independently verifies batch integrity during finalization:

```solidity
function finalizeBatch() external {
    // Reconstruct leaves from stored records
    bytes32[] memory leaves = new bytes32[](count);
    for (uint256 i = 0; i < count; i++) {
        leaves[i] = indexToRecords[startIndex + i].root;
    }

    // Verify against claimed root
    require(MerkleTree.verify(leaves, claimedRoot));
}
```

This prevents a malicious relayer from submitting a fraudulent Merkle root that doesn't match the actual queued entries. The contract uses its own stored data (not relayer-provided data) to reconstruct the tree.

## Reorg Protection

The L2 indexer rewinds by 100 blocks on startup:

```rust
const REWIND_BLOCKS: u64 = 100;
```

This protects against chain reorganizations that could cause the indexer to miss events. After a reorg:

1. The scanner re-processes the last 100 blocks.
2. Duplicate events are handled via UNIQUE constraints in SQLite.
3. The indexer converges to the correct chain state.

## Sequential Batch Ordering

The L2AnchoringManager enforces strict sequential batch ordering:

```solidity
require(startIndex == confirmedIndex);
require(pendingBatch.count == 0); // No overlapping batches
```

This prevents:
- **Gap attacks**: skipping queue entries to exclude specific timestamps.
- **Overlap attacks**: double-processing entries across multiple batches.
- **Reorder attacks**: processing entries out of order.

## Input Validation

### Batch Size Bounds

```solidity
uint256 constant MAX_BATCH_SIZE = 512;
require(count >= 1 && count <= MAX_BATCH_SIZE);
```

### Gas Limit Bounds

```solidity
uint256 constant MIN_GAS_LIMIT = 110_000;
uint256 constant MAX_GAS_LIMIT = 200_000;
require(gasLimit >= MIN_GAS_LIMIT && gasLimit <= MAX_GAS_LIMIT);
```

### Address Zero Checks

All address setters validate against `address(0)` to prevent accidental misconfiguration.

### Attestation Immutability

Submitted attestations are verified to be non-revocable and non-expiring:

```solidity
require(attestation.expirationTime == 0);
require(!attestation.revocable);
```

This ensures that once an attestation is used for L1 anchoring, it cannot be invalidated by the attester.

## Compare-and-Set Status Transitions

The relayer's batch state machine uses compare-and-set semantics in SQL:

```sql
UPDATE l1_batch
SET status = ?new_status, updated_at = ?now
WHERE id = ?id AND status = ?expected_status;
```

If the update affects zero rows (status changed concurrently), the operation is retried. This prevents race conditions in the unlikely event of concurrent relayer instances.

## Fail-Fast Error Handling

The journal implements a fatal error flag:

```rust
fatal_error: AtomicBool
```

Once set, all journal operations immediately return `Error::Fatal`. The calendar server initiates graceful shutdown rather than risk silent data corruption. This is preferable to attempting recovery from an unknown state.

## EAS Contract Addresses

UTS uses well-known, audited EAS contract addresses per chain:

| Chain | Address |
|-------|---------|
| Mainnet | `0xA1207F3BBa224E2c9c3c6D5aF63D0eb1582Ce587` |
| Scroll | `0xC47300428b6AD2c7D03BB76D05A176058b47E6B0` |
| Scroll Sepolia | `0xaEF4103A04090071165F78D45D83A0C0782c2B2a` |
| Sepolia | `0xC47300428b6AD2c7D03BB76D05A176058b47E6B0` |

These are hardcoded via a compile-time perfect hash map (`phf_map`), preventing runtime misconfiguration.
