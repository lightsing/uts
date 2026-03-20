// Package rpc provides clients for interacting with blockchain RPC endpoints.
//
// This package implements JSON-RPC clients for various blockchain networks
// used by the UTS protocol for timestamp verification:
//   - Bitcoin RPC client for block header and merkle root verification
//
// The Bitcoin client handles the byte reversal required when working with
// Bitcoin's display format (hashes are shown in reversed byte order).
package rpc
