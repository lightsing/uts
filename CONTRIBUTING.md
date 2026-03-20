# Contributing to UTS

Thank you for your interest in contributing to Universal Timestamps! This guide
will help you get started.

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md).
By participating, you are expected to uphold this code. Please report
unacceptable behavior to **light.tsing@gmail.com**.

## How to Contribute

### Reporting Bugs

- Check the [existing issues](https://github.com/lightsing/uts/issues) to avoid
  duplicates.
- Use the [bug report template](https://github.com/lightsing/uts/issues/new?template=bug_report.yml)
  and fill in as much detail as possible.

### Suggesting Features

- Open a [feature request](https://github.com/lightsing/uts/issues/new?template=feature_request.yml)
  describing the problem you want to solve and your proposed solution.

### Submitting Pull Requests

1. **Fork** the repository and create a branch from `main`.
2. **Make your changes** — keep PRs focused on a single concern.
3. **Add or update tests** for any changed functionality.
4. **Ensure all checks pass** (see [Development Setup](#development-setup)).
5. **Open a pull request** with a clear description of what and why.

## Development Setup

### Prerequisites

- **Rust** >= 1.94.0-nightly (e7d44143a 2025-12-24)
- **Cargo** >= 1.94.0-nightly (3861f60f6 2025-12-19)
- **pnpm** >= 10.26.2
- **Foundry** (for smart contract development)

### Building

```bash
# Rust crates
cargo build

# TypeScript packages
pnpm install
pnpm run build
```

### Testing

```bash
# Rust tests
cargo test

# TypeScript tests
pnpm run test

# Smart contract tests
forge test
```

### Linting & Formatting

```bash
# Rust
cargo fmt --check
cargo clippy --all-targets

# TypeScript
pnpm run lint
pnpm run format:check
```

## Project Structure

| Directory | Description |
| --- | --- |
| `crates/` | Rust crates (core library, servers, CLI) |
| `packages/` | TypeScript, Python, and Go SDKs |
| `contracts/` | Solidity smart contracts (Foundry) |
| `apps/web/` | Vue 3 web frontend |
| `book/` | Documentation (mdBook) |

### Key Crates

- **`uts-core`** — Core types, OTS codec, and verification logic
- **`uts-bmt`** — Binary Merkle Tree implementation
- **`uts-calendar`** — HTTP calendar server for digest submission
- **`uts-stamper`** — Batching engine for Merkle trees and EAS attestations
- **`uts-relayer`** — L2→L1→L2 relay service for cross-chain anchoring
- **`uts-cli`** — Command-line interface

### Ground Truth

The **Rust codebase** (`crates/`) is the authoritative reference implementation.
When implementing features in other languages, always check the Rust
implementation first.

## Licensing

Most of the project is dual-licensed under **MIT OR Apache-2.0**. The server
components (`uts-calendar` and `uts-relayer`) are licensed under **AGPL-3.0**.

By submitting a contribution, you agree that your contribution will be licensed
under the same terms as the component you are contributing to.

## Getting Help

- **Documentation**: [book.timestamps.now](https://book.timestamps.now/)
- **Issues**: [GitHub Issues](https://github.com/lightsing/uts/issues)
- **Discussions**: Use GitHub Issues for questions and discussions
