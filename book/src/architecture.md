# System Architecture Overview

UTS is organized as a Rust workspace of 11 crates plus a set of Solidity smart contracts. This chapter provides a bird's-eye view of the system, its components, and the two main pipelines.

## Component Diagram

```mermaid
graph TB
    subgraph User
        CLI[uts-cli]
    end

    subgraph "Calendar Server (L2)"
        CAL[uts-calendar]
        STAMP[uts-stamper]
        JOURNAL[uts-journal]
        KV[(RocksDB KV)]
        SQL[(SQLite)]
    end

    subgraph "Core Libraries"
        CORE[uts-core]
        BMT[uts-bmt]
        CONTRACTS[uts-contracts]
        SQLUTIL[uts-sql]
    end

    subgraph "Relayer Service"
        RELAYER[uts-relayer]
        RELAYDB[(SQLite)]
    end

    subgraph "Beacon Service"
        BEACON[uts-beacon-injector]
    end

    subgraph "Smart Contracts (On-Chain)"
        EAS[EAS Contract]
        EASHELPER[EASHelper.sol]
        MT[MerkleTree.sol]
        L1GW[L1AnchoringGateway]
        L2MGR[L2AnchoringManager]
        FEE[FeeOracle]
        NFT[NFTGenerator]
    end

    CLI -->|POST /digest| CAL
    CLI -->|GET /digest/commitment| CAL
    CAL --> JOURNAL
    CAL --> STAMP
    STAMP --> KV
    STAMP --> SQL
    STAMP -->|EAS.timestamp| EAS

    RELAYER -->|submitBatch| L1GW
    RELAYER -->|finalizeBatch| L2MGR
    L1GW -->|timestamp| EAS
    L1GW -->|cross-chain msg| L2MGR
    L2MGR --> FEE
    L2MGR --> NFT
    L2MGR --> MT

    BEACON -->|EAS.attest| EAS
    BEACON -->|submitForL1Anchoring| L2MGR
    BEACON -->|POST /digest| CAL

    STAMP --> BMT
    STAMP --> CORE
    STAMP --> CONTRACTS
    RELAYER --> BMT
    RELAYER --> CONTRACTS
    CAL --> CORE
    CLI --> CORE
```

## Component Inventory

| Crate | Purpose |
|-------|---------|
| `uts-bmt` | Binary Merkle Tree — flat-array, power-of-two, proof generation |
| `uts-core` | OTS codec (opcodes, timestamps, attestations), verification logic |
| `uts-journal` | RocksDB-backed write-ahead log with at-least-once delivery |
| `uts-calendar` | HTTP calendar server — accepts digests, serves proofs |
| `uts-stamper` | Batching engine — builds Merkle trees, submits attestations |
| `uts-cli` | Command-line tool — stamp, verify, inspect, upgrade |
| `uts-contracts` | Rust bindings for EAS and L2AnchoringManager contracts |
| `uts-relayer` | L2→L1→L2 relay service with batch state machine |
| `uts-beacon-injector` | Injects drand beacon randomness into the timestamping pipeline |
| `uts-sql` | SQLite utilities and Alloy type wrappers |
| `uts-contracts-sdk` | Smart contract SDK |

## Two Pipelines

UTS operates two complementary pipelines:

### Pipeline A: Calendar Timestamping (L2 Direct)

The fast path. User digests are batched into a Merkle tree and the root is timestamped directly on L2 (Scroll) via EAS. This provides low-latency, low-cost timestamps.

```mermaid
sequenceDiagram
    participant U as User (CLI)
    participant C as Calendar Server
    participant J as Journal
    participant S as Stamper
    participant EAS as EAS (L2)

    U->>C: POST /digest (hash)
    C->>C: Sign (EIP-191)
    C->>J: commit(commitment)
    C-->>U: OTS file + commitment

    loop Every batch interval
        S->>J: read entries
        S->>S: Build Merkle tree
        S->>EAS: timestamp(root)
        EAS-->>S: tx receipt
    end

    U->>C: GET /digest/{commitment}
    C->>C: Merkle proof + EASTimestamped
    C-->>U: Updated OTS file
```

### Pipeline B: L1 Anchoring (Cross-Chain)

The high-security path. L2 attestation roots are batched again and anchored on L1 Ethereum, providing L1-level finality guarantees. A relayer service orchestrates the cross-chain lifecycle.

```mermaid
sequenceDiagram
    participant U as User
    participant L2 as L2AnchoringManager
    participant R as Relayer
    participant L1 as L1AnchoringGateway
    participant EAS1 as EAS (L1)
    participant MSG as Scroll Messenger

    U->>L2: submitForL1Anchoring(attestationId)
    L2->>L2: Validate + queue

    R->>R: Pack batch (Merkle tree)
    R->>L1: submitBatch(root, startIndex, count)
    L1->>EAS1: timestamp(root)
    L1->>MSG: sendMessage(notifyAnchored)
    MSG->>L2: notifyAnchored(root, ...)

    R->>L2: finalizeBatch()
    L2->>L2: Verify Merkle root on-chain
    U->>L2: claimNFT(attestationId)
```

## Crate Dependency Graph

```mermaid
graph LR
    CLI[uts-cli] --> CORE[uts-core]
    CLI --> CONTRACTS[uts-contracts]

    CAL[uts-calendar] --> CORE
    CAL --> JOURNAL[uts-journal]
    CAL --> STAMPER[uts-stamper]

    STAMPER --> BMT[uts-bmt]
    STAMPER --> CORE
    STAMPER --> CONTRACTS
    STAMPER --> SQL_UTIL[uts-sql]

    RELAYER[uts-relayer] --> BMT
    RELAYER --> CONTRACTS
    RELAYER --> SQL_UTIL

    BEACON[uts-beacon-injector] --> CONTRACTS

    CONTRACTS --> CORE
```

## Beacon Injector

The beacon injector is an auxiliary service that injects [drand](https://drand.love/) randomness beacons into the timestamping pipeline. It submits beacon signatures to both the calendar server and the L1 anchoring pipeline, providing a continuous stream of publicly verifiable, unpredictable timestamps. See [Appendix A](./appendix-beacon.md) for details.
