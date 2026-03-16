---
layout: home

hero:
  name: UTS
  text: Universal Timestamps
  tagline: Decentralized timestamping protocol for verifiable proofs of existence
  image:
    src: /uts-logo.svg
    alt: UTS Logo
  actions:
    - theme: brand
      text: Get Started
      link: /guide/what-is-uts
    - theme: alt
      text: View on GitHub
      link: https://github.com/lightsing/uts

features:
  - icon: 🔐
    title: Cryptographic Proofs
    details: Create tamper-proof timestamps backed by blockchain consensus. Anyone can verify without trusting a third party.
  - icon: ⚡
    title: Zero Cost & Fast
    details: Public good infrastructure with no fees for users. Get timestamped in seconds, not hours.
---

## Quick Start

### Try our official web app

Visit [timestamps.now](https://timestamps.now) to timestamp your first file in seconds. No sign-up required!

### Via CLI

Install the CLI and timestamp your first file:

```bash
cargo install uts-cli --version 0.1.0-alpha.0 --locked
uts stamp myfile.txt
# wait ~10 seconds for the batch to be attested on-chain
uts upgrade myfile.txt.ots
uts verify myfile.txt
```

## How It Works

1. **Submit** - Send your data hash to a calendar server, get a pending timestamp in return
2. **Batch** - Your hash joins a Merkle tree with others
3. **Attest** - The tree root is timestamped on-chain via EAS
4. **Upgrade** - Upgrade the timestamp to self-contained.
5. **Verify** - Anyone can verify the timestamp proof

For implementation details, see the [Reference Book](https://book.timestamps.now).
