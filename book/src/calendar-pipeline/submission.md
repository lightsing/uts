# User Submission

This chapter describes the first stage of the calendar timestamping pipeline: how a user creates and submits a timestamp request.

## CLI: Hashing and Submission

The `uts stamp` command is the primary entry point for creating timestamps:

```bash
uts stamp --hasher keccak256 myfile.pdf
```

The CLI supports four hash algorithms: SHA-1, RIPEMD-160, SHA-256, and Keccak-256 (default).

### Workflow

1. **Hash the file** using the selected algorithm to produce a digest.
2. **Generate a nonce** for each file and build an internal Merkle tree (when stamping multiple files simultaneously).
3. **Submit** the tree root to one or more calendar servers.
4. **Merge responses** from multiple calendars into a single OTS file via `FORK` nodes.
5. **Write** the `.ots` detached timestamp file to disk.

### Multi-Calendar Quorum

The CLI can submit to multiple calendar servers for redundancy. Each server independently signs and stores the digest. The responses are merged:

```
digest
  └─ FORK
       ├─ Calendar A response (PendingAttestation)
       └─ Calendar B response (PendingAttestation)
```

This ensures that even if one calendar server goes offline, the timestamp can still be completed via the other.

## Calendar Server: POST /digest

The calendar server exposes a single endpoint for submissions:

```
POST /digest
Content-Type: application/octet-stream
Body: <raw digest bytes>
```

Validation: the digest must be ≤ 64 bytes.

### EIP-191 Signing

The server signs a binding message using EIP-191 (Ethereum personal sign):

```
\x19Ethereum Signed Message:\n<len><timestamp><digest>
```

Where:
- `timestamp` is the Unix time (seconds) of receipt.
- `digest` is the user's original hash.

The signature is encoded in ERC-2098 compact format (64 bytes instead of 65), producing an **undeniable** binding between the server's identity, the submission time, and the digest.

### Commitment Computation

The commitment is the value that becomes a leaf in the Merkle tree:

$$
\text{commitment} = \text{keccak256}\Big(\text{timestamp} \;\|\; \text{digest} \;\|\; \text{signature} \;\|\; \text{keccak256}(\text{digest})\Big)
$$

More precisely, the codec builds a `Timestamp` tree:

```
digest
  └─ PREPEND(timestamp_bytes)
       └─ APPEND(signature_bytes)
            └─ KECCAK256
                 └─ PendingAttestation { uri: "https://calendar/" }
```

The `KECCAK256` opcode produces the commitment — a deterministic 32-byte value that the user can later use to retrieve their proof.

### Journal Commit

The 32-byte commitment is written to the journal synchronously:

```rust
journal.try_commit(&commitment_bytes)?;
```

If the journal is full or in a fatal state, the server returns `503 Service Unavailable`. Otherwise, the entry is durably persisted before the HTTP response is sent.

### Response

The server returns:
- The encoded OTS bytes containing the pending timestamp tree.
- The 32-byte commitment for later retrieval via `GET /digest/{commitment}`.

The OTS file at this stage contains a `PendingAttestation` pointing back to the calendar server. The user must later poll the server to upgrade it to a confirmed attestation.

## Performance Optimizations

- **Thread-local bump allocator**: OTS encoding uses a per-thread bump allocator to avoid heap allocation overhead on the hot path.
- **Cached current time**: Unix seconds are cached globally and updated every second, avoiding repeated `clock_gettime` syscalls.
- **ERC-2098 compact signatures**: 64 bytes instead of 65, saving space in every OTS file.
