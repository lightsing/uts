# OpenTimestamps Codec

The OTS codec (`uts-core`) defines the binary format for timestamp proofs. It extends the original OpenTimestamps specification with new attestation types for Ethereum (EAS) while maintaining backward compatibility with the Bitcoin attestation format.

## OTS File Structure

A detached timestamp file (`.ots`) consists of three sections:

1. **Magic bytes + version** — file identification header.
2. **Digest header** — the hash algorithm and original digest value.
3. **Timestamp tree** — a directed acyclic graph of operations that transform the original digest into one or more attestation values.

The `DetachedTimestamp` struct wraps a `DigestHeader` and a `Timestamp` tree:

```rust
pub struct DigestHeader {
    kind: DigestOp,       // Which hash algorithm (SHA256, Keccak256, etc.)
    digest: [u8; 32],     // The original hash value (padded to 32 bytes)
}
```

## OpCode System

The codec defines a set of opcodes that describe transformations on byte sequences. Each opcode is a single byte:

### Data Opcodes

| OpCode | Tag | Description |
|--------|-----|-------------|
| `APPEND` | `0xf0` | Concatenate immediate data after the input |
| `PREPEND` | `0xf1` | Concatenate immediate data before the input |
| `REVERSE` | `0xf2` | Reverse the byte order |
| `HEXLIFY` | `0xf3` | Convert to ASCII hex representation |

### Digest Opcodes

| OpCode | Tag | Output Size | Description |
|--------|-----|-------------|-------------|
| `SHA1` | `0x02` | 20 bytes | SHA-1 hash |
| `RIPEMD160` | `0x03` | 20 bytes | RIPEMD-160 hash |
| `SHA256` | `0x08` | 32 bytes | SHA-256 hash |
| `KECCAK256` | `0x67` | 32 bytes | Keccak-256 hash |

### Control Opcodes

| OpCode | Tag | Description |
|--------|-----|-------------|
| `FORK` | `0xff` | Branch the proof into multiple paths |
| `ATTESTATION` | `0x00` | Terminal node — contains an attestation |

## The Timestamp Proof Tree

A `Timestamp` is a recursive structure that forms a proof tree (step graph):

```rust
pub enum Timestamp<A: Allocator = Global> {
    Step(Step<A>),
    Attestation(RawAttestation<A>),
}

pub struct Step<A: Allocator = Global> {
    op: OpCode,                    // Operation to execute
    data: Vec<u8, A>,              // Immediate data (for APPEND/PREPEND)
    input: OnceLock<Vec<u8, A>>,   // Cached computed input
    next: Vec<Timestamp<A>, A>,    // Child timestamps (1 normally, 2+ for FORK)
}
```

A single timestamp file can contain multiple attestations (e.g., both an EAS attestation and a Bitcoin attestation) connected via `FORK` nodes:

```
digest
  └─ PREPEND(timestamp)
       └─ APPEND(signature)
            └─ KECCAK256
                 ├─ [FORK: Calendar A path]
                 │    └─ APPEND(sibling₀)
                 │         └─ KECCAK256
                 │              └─ ATTESTATION(EASTimestamped)
                 └─ [FORK: Calendar B path]
                      └─ ATTESTATION(PendingAttestation)
```

## Attestation Types

Each attestation is identified by an 8-byte tag and carries type-specific data:

### PendingAttestation (`0x83dfe30d2ef90c8e`)

Indicates the timestamp is not yet confirmed. Contains a URI pointing to the calendar server where the user can retrieve the completed proof.

```rust
pub struct PendingAttestation<'a> {
    uri: Cow<'a, str>,  // e.g., "https://calendar.example.com/digest/<commitment>"
}
```

URI validation: max 1000 bytes, restricted character set (`a-zA-Z0-9.-_/:`).

### EASAttestation (`0x8bf46bf4cfd674fa`)

A confirmed attestation on EAS with a specific UID.

```rust
pub struct EASAttestation {
    chain: Chain,  // Ethereum chain (mainnet, Scroll, etc.)
    uid: B256,     // 32-byte attestation UID
}
```

Encoded as: `chain_id (u64) || uid (32 bytes)`.

### EASTimestamped (`0x5aafceeb1c7ad58e`)

A lighter attestation that records only the chain where the timestamp was created. The on-chain lookup uses the computed commitment value to find the timestamp.

```rust
pub struct EASTimestamped {
    chain: Chain,  // Only the chain identifier
}
```

Encoded as: `chain_id (u64)`.

### BitcoinAttestation (`0x0588960d73d71901`)

Compatibility with the original OpenTimestamps Bitcoin anchoring.

```rust
pub struct BitcoinAttestation {
    height: u32,  // Bitcoin block height
}
```

## Commitment Computation

When a user submits a digest to a calendar server, the server computes a **commitment** — a deterministic value that binds the digest to the submission time and the server's identity:

$$
\text{commitment} = \text{keccak256}(\text{prepend}(ts) \;\|\; \text{append}(sig) \;\|\; \text{keccak256}(digest))
$$

Where:
- $ts$ is the Unix timestamp (seconds) of receipt.
- $sig$ is the server's EIP-191 signature over `timestamp || digest`.
- $digest$ is the user's original hash.

This commitment becomes the leaf in the Merkle tree.

## Finalization

The `Timestamp::finalize(input)` method walks the proof tree and computes the value at each node by executing its opcode:

1. For a `Step`: execute the opcode on the input (with immediate data if applicable), then finalize all children with the output.
2. For a `FORK`: finalize all children with the same input (the proof branches).
3. For an `Attestation`: store the input as the attestation's value (the commitment that should match on-chain).

Finalization uses `OnceLock` for caching — once a node's input is computed, it is stored and never recomputed. Conflicting inputs (from multiple paths) produce a `FinalizationError`.

## Iterating Attestations

The `Timestamp::attestations()` method returns a depth-first iterator over all `RawAttestation` nodes in the tree. This is used during verification to extract and check each attestation independently.

The mutable variant `pending_attestations_mut()` allows upgrading pending attestations to confirmed ones (e.g., replacing a `PendingAttestation` with an `EASTimestamped` after the calendar server confirms the timestamp).
