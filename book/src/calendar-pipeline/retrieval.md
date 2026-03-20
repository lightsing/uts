# Proof Retrieval

After the on-chain attestation succeeds, users can retrieve their completed timestamp proof from the calendar server.

## Retrieval Endpoint

```
GET /digest/{commitment}
```

Where `{commitment}` is the 32-byte hex commitment returned during submission.

## Lookup Flow

### Step 1: Leaf → Root

The server looks up the commitment in the RocksDB KV store:

```rust
let root = kv_db.get_root_for_leaf(commitment)?;
```

If the commitment is not found, the entry hasn't been batched yet — return `404 Not Found`.

### Step 2: Check Attestation Status

The server queries SQLite for the attestation result:

```rust
let result = get_attestation_result(&pool, root).await?;
```

| Status | HTTP Response |
|--------|--------------|
| `pending` | `404 Not Found` (not yet attested) |
| `max_attempts_exceeded` | `500 Internal Server Error` |
| `success` | Continue to proof construction |

### Step 3: Merkle Proof Reconstruction

The server loads the full Merkle tree from KV storage and generates a proof for the user's leaf:

```rust
let tree = kv_db.load_trie::<Keccak256>(root)?;
let proof_iter = tree.get_proof_iter(&commitment)?;
```

The proof is a sequence of `(NodePosition, Hash)` pairs that the user needs to reconstruct the root from their leaf.

### Step 4: Build Timestamp Tree

The proof is encoded as an OTS `Timestamp` tree with the Merkle proof steps and a terminal `EASTimestamped` attestation:

```
commitment
  └─ APPEND(sibling₀)    or  PREPEND(sibling₀)
       └─ KECCAK256
            └─ APPEND(sibling₁)  or  PREPEND(sibling₁)
                 └─ KECCAK256
                      └─ ...
                           └─ EASTimestamped { chain: scroll }
```

Each sibling in the Merkle proof becomes either an `APPEND` or `PREPEND` operation, depending on whether the target node is the left or right child (i.e., the `NodePosition`). After each append/prepend, a `KECCAK256` operation computes the parent hash.

### Step 5: Encode and Return

The timestamp tree is encoded to OTS binary format and returned with caching headers:

```
Cache-Control: public, immutable
```

Once an attestation is confirmed on-chain, the proof is **immutable** — the same commitment will always produce the same response. This allows aggressive client-side and CDN caching.

## Complete Response Structure

The user's final `.ots` file (after merging the retrieval response with their original submission) looks like:

```
digest
  └─ PREPEND(timestamp)
       └─ APPEND(signature)
            └─ KECCAK256                    ← commitment (leaf)
                 └─ APPEND(sibling₀)
                      └─ KECCAK256
                           └─ PREPEND(sibling₁)
                                └─ KECCAK256
                                     └─ EASTimestamped { chain_id }
```

The user now has a self-contained proof that:
1. Their digest was received at a specific time (via the timestamp + signature).
2. The commitment was included in a specific Merkle tree (via the proof path).
3. The Merkle root was recorded on-chain (via the EASTimestamped attestation).
