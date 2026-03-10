# What is UTS?

**Universal Timestamps (UTS)** is a decentralized timestamping protocol that enables anyone to create cryptographic, publicly verifiable proofs that data existed at a specific point in time.

## The Problem

Consider a common scenario: you write a document, invent a design, or generate a dataset. Later, someone disputes when that data came into existence. How do you *prove* the data existed at a certain time without relying on a trusted third party?

Traditional approaches — notary stamps, trusted servers, email timestamps — all share a common weakness: they depend on a single entity that must be trusted not to backdate, forge, or lose records. A compromised notary or a deleted email server destroys the proof.

## The Analogy: Digital Notarization

Think of UTS as a **digital notary** backed by a public blockchain:

1. You bring your document (any data) to the notary.
2. The notary doesn't read your document — it only sees a cryptographic **hash** (a fixed-size fingerprint).
3. The notary records that hash into a **public, append-only ledger** that anyone can audit.
4. Later, anyone can verify the timestamp by re-hashing the original data and checking the ledger.

Unlike a physical notary, UTS requires no trust in any single party. The ledger is a blockchain — immutable, publicly verifiable, and decentralized.

## Why Blockchain?

Blockchains provide three properties that are ideal for timestamping:

- **Immutability** — once a transaction is confirmed, it cannot be altered or removed.
- **Public verifiability** — anyone can independently verify that a hash was recorded at a given block height.
- **No trusted third party** — the security guarantee comes from the consensus mechanism, not from any single operator.

## OpenTimestamps Heritage

UTS extends the [OpenTimestamps](https://opentimestamps.org/) protocol, which pioneered blockchain-based timestamping on Bitcoin. OpenTimestamps introduced several key ideas:

- A compact **binary codec** (`.ots` files) that encodes hash operations as a directed acyclic graph of opcodes.
- **Calendar servers** that aggregate many timestamp requests and batch them into a single on-chain transaction.
- **Merkle tree batching** — thousands of timestamps share a single blockchain transaction by constructing a Merkle tree and recording only the root on-chain.

UTS builds on this foundation and extends it to **Ethereum** via the [Ethereum Attestation Service (EAS)](https://attest.org/), using a dual-layer architecture across **L2 (Scroll)** and **L1 (Ethereum mainnet)**.

## Key Insight: Cost Amortization

A single Ethereum transaction costs gas regardless of whether it timestamps one hash or one thousand. UTS exploits this by **batching**: a calendar server collects many user digests, builds a Merkle tree from them, and records only the 32-byte Merkle root on-chain. Each user receives a Merkle proof that links their specific hash to that on-chain root.

The result: the per-timestamp cost drops by orders of magnitude, making cryptographic timestamping practical for everyday use.

## What You'll Learn

This book walks through the UTS architecture from first principles:

- **Chapter 2** gives a high-level system overview and introduces all components.
- **Chapter 3** explains the core data structures: Merkle trees, the OTS codec, and the journal.
- **Chapter 4** traces the calendar timestamping pipeline end-to-end.
- **Chapter 5** covers the L1 anchoring pipeline for cross-chain security.
- **Chapter 6** describes the storage architecture.
- **Chapter 7** discusses security considerations.
- **Appendix A** explains the drand beacon injector.
