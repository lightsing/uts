// Package crypto provides cryptographic hash functions for the UTS protocol.
//
// This package wraps standard and Ethereum-compatible hash functions used
// throughout the UTS protocol for:
//   - Timestamp operations (SHA256, Keccak256 steps)
//   - Merkle tree construction
//   - Digest computation
//
// Supported hash functions:
//   - SHA256: Standard SHA-256 hash function (32 bytes output)
//   - Keccak256: Ethereum's Keccak-256 hash function (32 bytes output)
package crypto
