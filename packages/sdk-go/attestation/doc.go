// Package attestation provides verification functions for timestamp attestations.
//
// This package implements verification logic for various attestation types
// including Bitcoin block headers and Ethereum attestation service attestations.
//
// Bitcoin Attestation Verification:
//
// Bitcoin attestations are verified by:
// 1. Fetching the block hash at the specified height via RPC
// 2. Fetching the block header using the hash
// 3. Comparing the merkle root (reversed) with the digest
//
// The merkle root byte reversal is necessary because Bitcoin displays
// hashes in little-endian format.
package attestation
