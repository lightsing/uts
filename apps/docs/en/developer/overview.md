# Architecture Overview

UTS implements a dual-layer timestamping architecture.

## High-Level Flow

```
User → Calendar Server → Merkle Tree → EAS Attestation → Blockchain
```

## Components

### Calendar Server

HTTP server that:

- Accepts digest submissions
- Batches digests into Merkle trees
- Returns timestamp proofs

### Stamper

Batching engine that:

- Builds Merkle trees from pending digests
- Submits attestations via EAS
- Manages calendar state

### Relayer

Cross-chain service that:

- Anchors L2 roots to L1
- Bridges attestation proofs back to L2

## Dual-Layer Architecture

### L2 Direct Path (Fast)

1. User submits digest to calendar
2. Stamper batches into Merkle tree
3. Root attested on L2 via EAS (Scroll)
4. User receives proof in seconds

### L1 Anchoring Path (Secure)

1. L2 attestation roots collected
2. Batched into L1 Merkle tree
3. Anchored on Ethereum mainnet
4. Provides maximum security

## Data Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│    User     │────▶│  Calendar   │────▶│   Stamper   │
└─────────────┘     └─────────────┘     └─────────────┘
                                               │
                                               ▼
                                        ┌─────────────┐
                                        │     EAS     │
                                        └─────────────┘
                                               │
                          ┌────────────────────┴────────────────────┐
                          ▼                                         ▼
                   ┌─────────────┐                          ┌─────────────┐
                   │  L2 (Scroll)│                          │  L1 Relayer │
                   └─────────────┘                          └─────────────┘
                                                                   │
                                                                   ▼
                                                            ┌─────────────┐
                                                            │  Ethereum   │
                                                            └─────────────┘
```

## Learn More

For detailed implementation, see the [Reference Book](https://book.timestamps.now):

- [System Architecture](https://book.timestamps.now/architecture.html)
- [Calendar Pipeline](https://book.timestamps.now/calendar-pipeline/submission.html)
- [L1 Anchoring](https://book.timestamps.now/l1-anchoring/contracts.html)
