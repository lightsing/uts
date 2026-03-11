# UTS Go SDK

A Go SDK for the [Universal Timestamps (UTS)](https://book.timestamps.now/) protocol.

UTS is a superset of OpenTimestamps that batches user-submitted digests into Merkle trees and anchors the roots on-chain via EAS attestations, providing trustless, verifiable timestamps without relying on a single trusted calendar server.

## Installation

```bash
go get github.com/uts-dot/sdk-go
```

Requires Go 1.24 or later.

## Quick Start

```go
package main

import (
    "context"
    "crypto/sha256"
    "fmt"
    "log"

    "github.com/uts-dot/sdk-go"
    "github.com/uts-dot/sdk-go/types"
)

func main() {
    ctx := context.Background()

    sdk := uts.NewSDK(
        uts.WithCalendars("https://lgm1.test.timestamps.now/"),
    )

    data := []byte("Hello, UTS!")
    hash := sha256.Sum256(data)

    header := types.NewDigestHeader(types.DigestSHA256, hash[:])
    stamps, err := sdk.Stamp(ctx, []*types.DigestHeader{header})
    if err != nil {
        log.Fatal(err)
    }

    result, err := sdk.Verify(ctx, stamps[0])
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("Status: %s\n", result.Status)
    for _, att := range result.Attestations {
        fmt.Printf("  %s\n", att)
    }
}
```

## Features

- **Timestamp Creation**: Submit digests to calendar servers and receive pending timestamps
- **Timestamp Verification**: Verify attestation proofs against Bitcoin block headers and EAS attestations
- **Pending Attestation Upgrade**: Convert pending attestations to confirmed ones
- **Binary Codec**: Encode and decode timestamps in the OpenTimestamps binary format
- **Binary Merkle Tree**: Efficient Merkle tree operations for batching digests

## API Overview

### SDK Constructor

```go
sdk := uts.NewSDK(
    uts.WithCalendars("https://calendar1.example.com/", "https://calendar2.example.com/"),
    uts.WithTimeout(30 * time.Second),
    uts.WithQuorum(2),
    uts.WithNonceSize(32),
    uts.WithHashAlgorithm(uts.HashKeccak256),
    uts.WithBitcoinRPC(btcClient),
    uts.WithEthereumRPC(chainID, rpcURL),
)
```

### Core Methods

| Method                             | Description                        |
| ---------------------------------- | ---------------------------------- |
| `Stamp(ctx, headers)`              | Submit digests to calendar servers |
| `Verify(ctx, stamp)`               | Verify a detached timestamp        |
| `Upgrade(ctx, stamp, keepPending)` | Upgrade pending attestations       |

### Types

| Type                 | Description                                |
| -------------------- | ------------------------------------------ |
| `DigestHeader`       | Digest algorithm and hash value            |
| `DetachedTimestamp`  | Header and timestamp proof                 |
| `Timestamp`          | Sequence of operations/steps               |
| `VerificationResult` | Verification status and attestations       |
| `AttestationStatus`  | Individual attestation verification result |

### Attestation Types

| Type                 | Tag                  | Description                   |
| -------------------- | -------------------- | ----------------------------- |
| `BitcoinAttestation` | `0x0588960d73d71901` | Bitcoin block height          |
| `PendingAttestation` | `0x83dfe30d2ef90c8e` | Pending calendar URI          |
| `EASAttestation`     | `0x8bf46bf4cfd674fa` | EAS attestation UID and chain |
| `EASTimestamped`     | `0x5aafceeb1c7ad58e` | EAS timestamp attestation     |

### Binary Codec

```go
import "github.com/uts-dot/sdk-go/codec"

encoded, err := codec.EncodeDetachedTimestamp(stamp)

stamp, err := codec.DecodeDetachedTimestamp(encoded)
```

### Hash Operations

```go
import "github.com/uts-dot/sdk-go/crypto"

hash := crypto.SHA256(data)
hash := crypto.Keccak256(data)
```

### Merkle Tree

```go
import "github.com/uts-dot/sdk-go/crypto"

tree := crypto.NewMerkleTree(leaves)
root := tree.Root()
proof, err := tree.GetProof(leaf)
```

## Compatibility

The binary format is compatible with the TypeScript and Python SDKs. The Rust implementation in `crates/core` is the ground truth for:

- Binary encoding/decoding format
- Attestation type definitions
- Error codes and handling
- Merkle tree algorithm details

When implementing features or debugging issues, always cross-reference with the Rust code.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](https://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](https://opensource.org/licenses/MIT))

at your option.
