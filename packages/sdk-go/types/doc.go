// Package types defines core timestamp types for the UTS protocol.
//
// A Timestamp represents a sequence of operations (steps) that transform an input
// digest, ultimately leading to an attestation. Each step either transforms the
// current value or branches (FORK).
//
// Operation types:
//   - Transform ops: APPEND, PREPEND, REVERSE, HEXLIFY
//   - Hash ops: SHA1, RIPEMD160, SHA256, KECCAK256
//   - Control ops: ATTESTATION, FORK
package types
