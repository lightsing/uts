# What is UTS?

**Universal Timestamps (UTS)** is a superset of the [OpenTimestamps][ots] protocol,
designed to provide a decentralized, trustless way to prove that data existed
prior a specific point in time.

By leveraging the security and immutability of public blockchains, UTS allows
anyone to create cryptographic proofs of existence that are publicly verifiable
without relying on a trusted third party.

## Public Good Infrastructure

The cost of creating on-chain attestations is covered by the protocol operators,
making UTS free for users.

### The Calendar Server

The calendar server batches incoming timestamp requests into Merkle trees and
periodically attests the tree roots on-chain.

This aggregation allows UTS to batch thousands of timestamps into a single
on-chain transaction, making it cost-effective.

Current operational calendar servers:

- [lgm1.calendar.test.timestamps.now](https://lgm1.calendar.test.timestamps.now) (Testnet)

## Learn More

- [Stamp via CLI](/guide/stamp-via-cli) - Get your first timestamp from CLI
- [Architecture Overview](/developer/overview) — How the system works
- [Reference Book](https://book.timestamps.now) — Detailed technical documentation

[ots]: https://opentimestamps.org/
