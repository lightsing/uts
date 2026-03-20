# Storage Architecture

UTS uses a three-layer storage strategy, choosing the right technology for each workload's access pattern and durability requirements.

## Storage Overview

| Layer | Technology | Component | Purpose | Data Stored |
|-------|-----------|-----------|---------|-------------|
| Journal | RocksDB | Calendar Server | High-throughput WAL | Pending digest commitments |
| KV Store | RocksDB | Calendar Server | Merkle tree storage | Trees + leaf→root mappings |
| SQL Store | SQLite | Calendar Server (Stamper) | Attestation metadata | Pending attestations + attempt history |
| Relayer DB | SQLite | Relayer Service | Event indexing + batch state | Cursors, batches, costs, event logs |

## Journal (RocksDB)

**Purpose**: Durable buffer between HTTP handler and stamper.

**Access pattern**: Append-only writes, sequential reads, bulk deletes.

RocksDB is ideal here because:
- Synchronous writes guarantee durability before HTTP response.
- Sequential key layout (monotonic u64 indices) enables efficient range scans.
- Bulk deletes on commit are efficient via RocksDB's compaction.

**Column families**:
- `entries` — entry data keyed by write index.
- `meta` — `write_index` and `consumed_index` metadata.

**Capacity**: Configurable (default: 1,048,576 entries). Back-pressure via `Error::Full` when capacity is reached.

See [Journal / WAL](./core-primitives/journal.md) for implementation details.

## KV Store (RocksDB)

**Purpose**: Store Merkle trees and leaf→root mappings for proof retrieval.

**Access pattern**: Point lookups by 32-byte hash keys.

Two types of entries:

| Key | Value | Size |
|-----|-------|------|
| Leaf hash (32B) | Root hash (32B) | 64 bytes per entry |
| Root hash (32B) | Serialized tree | Variable (depends on leaf count) |

The KV store uses RocksDB's default column family with `DB::open_default()`. Trees are serialized as raw byte arrays via `MerkleTree::as_raw_bytes()` for zero-copy storage and retrieval.

**Retrieval logic** (`DbExt` trait):
- `get_root_for_leaf(leaf)`: Returns the root hash for a given commitment.
- `load_trie(root)`: Deserializes the full Merkle tree for proof generation.

For single-leaf trees, the leaf itself is the root — the tree serialization is stored directly and detected by value length (≠ 32 bytes).

## SQL Store — Stamper (SQLite)

**Purpose**: Track attestation lifecycle and transaction attempts.

**Schema**:

```sql
-- Pending attestation records
CREATE TABLE pending_attestations (
    id          INTEGER PRIMARY KEY,
    trie_root   TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    result      TEXT NOT NULL DEFAULT 'pending'
    -- result: 'pending' | 'success' | 'max_attempts_exceeded'
);

-- Individual transaction attempts
CREATE TABLE attest_attempts (
    id              INTEGER PRIMARY KEY,
    attestation_id  INTEGER NOT NULL REFERENCES pending_attestations(id),
    chain_id        TEXT NOT NULL,
    tx_hash         TEXT,
    block_number    TEXT,
    created_at      INTEGER NOT NULL
);
```

**Encoding**: Large integers and 32-byte hashes are stored as text (hex or decimal strings) via a `TextWrapper<T>` pattern to improve human-readability. The performance impact is minimal and hence is an acceptable trade-off.

## Relayer DB (SQLite)

**Purpose**: Event indexing, batch lifecycle management, and cost tracking.

The relayer database is more complex, with 10 tables across 3 migrations:

### Indexer Tables

```sql
indexer_cursors       -- Track last-indexed block per event type
eth_block             -- Block metadata for indexed events
eth_transaction       -- Transaction metadata
eth_log               -- Log metadata
```

These four tables form a normalized chain of custody: `block → transaction → log → event-specific table`.

### Event Tables

```sql
l1_anchoring_queued   -- L1AnchoringQueued events (user submissions)
l1_batch_arrived      -- L1BatchArrived events (cross-chain notifications)
l1_batch_finalized    -- L1BatchFinalized events (batch completions)
```

### Batch Management

```sql
l1_batch              -- Batch lifecycle state machine
                      -- Columns: start_index, count, root, l1_tx_hash, l2_tx_hash, status
                      -- Status: Collected → L1Sent → L1Mined → L2Received
                      --       → L2FinalizeTxSent → L2Finalized
```

### Cost Tracking

```sql
tx_receipt            -- Gas usage and pricing per transaction
batch_fee             -- Per-batch cost breakdown (L1 gas, L2 gas, cross-chain fee)
```

## Why This Split?

| Concern | RocksDB | SQLite |
|---------|---------|--------|
| High-throughput sequential writes | Excellent | Adequate |
| Point lookups by hash | Excellent | Good (with index) |
| Complex queries (JOINs, aggregations) | Not supported | Excellent |
| Relational integrity (foreign keys) | Not supported | Built-in |
| Schema evolution (migrations) | Manual | SQLx migrations |

RocksDB handles the **hot path** (journal writes, tree storage) where throughput matters. SQLite handles the **metadata path** (attestation tracking, event indexing) where query flexibility matters.

This split avoids forcing either technology into a role it's not designed for.
