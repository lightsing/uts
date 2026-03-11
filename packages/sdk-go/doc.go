// Package uts provides a Go SDK for the Universal Timestamps (UTS) protocol.
//
// UTS is a superset of OpenTimestamps that batches user-submitted digests into
// Merkle trees and anchors the roots on-chain via EAS attestations, providing
// trustless, verifiable timestamps without relying on a single trusted calendar server.
//
// The SDK provides client-side interaction with the UTS protocol, including:
//   - Timestamp creation and verification
//   - EAS attestation handling
//   - Binary Merkle Tree operations
//   - Calendar server communication
//
// For more information, see https://book.timestamps.now/
package uts
