---
seo:
  title: Universal Timestamps
  description: Decentralized timestamping protocol with EAS attestations. Create cryptographic, publicly verifiable proofs that data existed at a specific point in time.
---

::u-page-hero{class="dark:bg-gradient-to-b from-violet-950 to-neutral-950"}
---
orientation: horizontal
---
#top
:hero-background

#title
Decentralized Timestamps, [Verifiable Forever]{.text-primary}.

#description
UTS extends OpenTimestamps with Ethereum Attestation Service (EAS) integration. Batch Merkle trees, anchor on L2, and secure with L1 finality. The universe's most precise clocks for your data.

#links
  :::u-button
  ---
  to: /getting-started
  size: xl
  trailing-icon: i-lucide-arrow-right
  ---
  Get started
  :::

  :::u-button
  ---
  icon: i-simple-icons-github
  color: neutral
  variant: outline
  size: xl
  to: https://github.com/lightsing/uts
  target: _blank
  ---
  View on GitHub
  :::

#default
  :::prose-pre
  ---
  code: |
    // Install the CLI
    cargo install uts-cli

    # Timestamp a file
    uts stamp document.pdf

    # Verify timestamp
    uts verify document.pdf.ots
  filename: Terminal
  ---

  ```bash
  # Install the CLI
  cargo install uts-cli

  # Timestamp a file
  uts stamp document.pdf

  # Verify timestamp
  uts verify document.pdf
  ```
  :::
::

::u-page-section{class="dark:bg-neutral-950"}
#title
Why UTS?

#links
  :::u-button
  ---
  color: neutral
  size: lg
  target: _blank
  to: https://attest.org/
  trailingIcon: i-lucide-arrow-right
  variant: subtle
  ---
  Learn about EAS
  :::

#features
  :::u-page-feature
  ---
  icon: i-lucide-shield-check
  ---
  #title
  Trustless Verification

  #description
  No trusted third parties. Timestamps are anchored on Ethereum via EAS attestations, providing immutable, publicly verifiable proofs.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-layers
  ---
  #title
  Dual-Layer Security

  #description
  Fast L2 timestamps on Scroll for immediate confirmation. Optional L1 Ethereum anchoring for maximum finality guarantees.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-git-merge
  ---
  #title
  Merkle Tree Batching

  #description
  Thousands of digests share a single on-chain transaction. Amortized costs make cryptographic timestamping practical for everyone.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-zap
  ---
  #title
  OpenTimestamps Compatible

  #description
  Builds on the proven OTS binary format. Existing tools and workflows integrate seamlessly with enhanced Ethereum support.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-code
  ---
  #title
  Multi-Language SDKs

  #description
  Rust, TypeScript, Python, and Go SDKs. Integrate timestamping into any project with familiar, well-documented libraries.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-clock
  ---
  #title
  Beacon Integration

  #description
  Drand randomness injection for continuous, unpredictable timestamps. Perfect for applications requiring public randomness.
  :::
::

::u-page-section{class="dark:bg-neutral-950"}
#title
How It Works

#features
  :::u-page-feature
  ---
  icon: i-lucide-upload
  ---
  #title
  1. Submit Digest

  #description
  Send your data's SHA256 hash to a calendar server. Your original data never leaves your machine—only the cryptographic fingerprint.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-tree-deciduous
  ---
  #title
  2. Merkle Tree Batching

  #description
  The calendar aggregates thousands of digests into a Binary Merkle Tree. You receive an OTS file with your Merkle proof path.
  :::

  :::u-page-feature
  ---
  #icon: i-lucide-badge-check
  ---
  #title
  3. EAS Attestation

  #description
  The Merkle root is timestamped on L2 (Scroll) via Ethereum Attestation Service. Fast confirmation, low cost, immutable record.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-link
  ---
  #title
  4. L1 Anchoring (Optional)

  #description
  L2 attestation roots are batched and anchored on Ethereum L1. Cross-chain relay ensures finality on the main chain.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-search-check
  ---
  #title
  5. Verify Anytime

  #description
  Re-hash your data, verify the Merkle proof, and check the on-chain attestation. Trustless verification, forever.
  :::
::

::u-page-section{class="dark:bg-neutral-950"}
#title
Built with Modern Technology

#links
  :::u-button
  ---
  color: neutral
  size: lg
  target: _blank
  to: https://www.rust-lang.org/
  trailingIcon: i-lucide-arrow-right
  variant: subtle
  ---
  Explore the Stack
  :::

#features
  :::u-page-feature
  ---
  icon: i-simple-icons-rust
  ---
  #title
  Rust Backend

  #description
  High-performance, memory-safe Rust implementation. Tokio async runtime, Axum web framework, RocksDB storage.
  :::

  :::u-page-feature
  ---
  icon: i-simple-icons-ethereum
  ---
  #title
  Ethereum Ecosystem

  #description
  Deep EAS integration, Foundry smart contracts, Alloy bindings. Native support for L2 (Scroll) and L1 (Ethereum).
  :::

  :::u-page-feature
  ---
  icon: i-simple-icons-typescript
  ---
  #title
  TypeScript SDK

  #description
  Full-featured TypeScript/JavaScript SDK. Browser and Node.js compatible with complete type safety.
  :::
::

::u-page-section{class="dark:bg-gradient-to-b from-neutral-950 to-violet-950"}
  :::u-page-c-t-a
  ---
  links:
    - label: Get started
      to: '/getting-started'
      trailingIcon: i-lucide-arrow-right
    - label: View on GitHub
      to: 'https://github.com/lightsing/uts'
      target: _blank
      variant: subtle
      icon: i-simple-icons-github
  title: Ready to timestamp your data?
  description: Join the decentralized timestamping revolution. Start with the CLI or integrate the SDK into your application.
  class: dark:bg-neutral-950
  ---

  :stars-bg
  :::
::
