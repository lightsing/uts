# Journal / WAL

The journal (`uts-journal`) is a RocksDB-backed write-ahead log that sits between the calendar server's HTTP handler and the stamper's batching engine. It provides **at-least-once delivery** semantics and crash recovery for incoming timestamp requests.

## Why a WAL?

The calendar server and stamper operate at different speeds and cadences:

- The HTTP handler accepts user digests one at a time, potentially hundreds per second.
- The stamper batches digests into Merkle trees on a configurable interval (default: every 10 seconds).

Without a durable buffer between them, a crash between receiving a digest and building the next batch would lose user data. The journal solves this by persisting every entry to disk synchronously before acknowledging the HTTP request.

## Architecture

```
     ┌──────────────┐         ┌──────────┐         ┌─────────┐
     │ HTTP Handler │──commit──▶│ Journal  │──read───▶│ Stamper │
     │  (writer)    │         │ (RocksDB)│         │(reader) │
     └──────────────┘         └──────────┘         └─────────┘
                                   │
                              ┌────┴────┐
                              │CF_ENTRIES│  ← entry data
                              │CF_META   │  ← write/consumed indices
                              └─────────┘
```

## RocksDB Column Families

The journal uses two RocksDB column families:

| Column Family | Key | Value | Purpose |
|--------------|-----|-------|---------|
| `CF_ENTRIES` (`"entries"`) | `write_index` (u64 big-endian) | Raw entry bytes | Stores the actual digest commitments |
| `CF_META` (`"meta"`) | `0x00` or `0x01` | u64 little-endian | Stores `write_index` and `consumed_index` |

## Writer / Reader Pattern

The journal enforces a strict concurrency model:

- **One writer** (the HTTP handler), serialized by a `Mutex`.
- **One exclusive reader** (the stamper), enforced by an `AtomicBool` flag.

### Write Path

```rust
// From crates/journal/src/lib.rs
pub fn try_commit(&self, data: &[u8]) -> Result<(), Error>
```

1. Acquire write lock.
2. Check capacity: if `write_index - consumed_index >= capacity`, return `Error::Full`.
3. Write entry + updated `write_index` atomically via `WriteBatch`.
4. Update in-memory `write_index` (`AtomicU64`).
5. Notify the consumer (stamper) that new data is available.

Every commit is a **synchronous** RocksDB write. The in-memory `write_index` always matches the durable state — there is no separate "flush" step.

### Read Path

The `JournalReader` maintains a local cursor independent of the journal's `consumed_index`:

```rust
// From crates/journal/src/reader.rs
reader.wait_at_least(min).await;   // Async wait for entries
let entries = reader.read(max);     // Fetch into internal buffer
// ... process entries ...
reader.commit();                    // Advance consumed_index
```

The critical invariant: entries are only deleted from RocksDB when the reader calls `commit()`. This ensures that if the stamper crashes after reading but before building the Merkle tree, the entries survive for re-processing on restart.

## Capacity Management

The journal has a fixed capacity (default: 1,048,576 entries in the calendar configuration). When the journal is full (`write_index - consumed_index >= capacity`), the HTTP handler receives a `503 Service Unavailable` response rather than blocking.

This back-pressure mechanism prevents unbounded memory growth and signals to clients that the server is temporarily overloaded.

## Crash Recovery

On startup, the journal reads `write_index` and `consumed_index` from `CF_META` and validates the invariant:

$$
\text{consumed\_index} \leq \text{write\_index}
$$

If both are zero (fresh database), the journal starts empty. Otherwise, it resumes from where it left off — any entries between `consumed_index` and `write_index` are re-delivered to the reader.

## Fatal Errors

If RocksDB encounters an unrecoverable error (e.g., disk corruption), the journal sets an `AtomicBool` fatal error flag. All subsequent operations immediately return `Error::Fatal`, and the calendar server initiates graceful shutdown.

This fail-fast behavior prevents silent data loss — the operator must investigate and fix the storage issue before the server can restart.

## Async Coordination

The journal uses a waker-based notification system for efficient async coordination:

1. When the reader calls `wait_at_least(n)` and fewer than `n` entries are available, it registers a waker.
2. When the writer commits a new entry, it checks for a registered waker and wakes the reader task.
3. This avoids busy-polling and integrates cleanly with Tokio's async runtime.
