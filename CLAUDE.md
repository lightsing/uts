# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Development Commands

### Rust Backend

- **Build all crates**: `cargo build`
- **Run tests**: `cargo test`
- **Run specific test**: `cargo test test_name`
- **Install CLI**: `cargo install --path crates/cli`
- **Check formatting**: `cargo fmt --check`
- **Format code**: `cargo fmt`

### TypeScript/JavaScript Frontend & SDK

- **Install dependencies**: `pnpm install`
- **Build all packages**: `pnpm run build`
- **Run all tests**: `pnpm run test`
- **Lint code**: `pnpm run lint`
- **Type check**: `pnpm run typecheck`
- **Format code**: `pnpm run format`

### Smart Contracts (Foundry)

- **Compile contracts**: `forge build`
- **Run contract tests**: `forge test`
- **Run specific test**: `forge test --match-test test_name`
- **Deploy scripts**: `forge script <script_name>`

### Monorepo Commands

- **Build entire monorepo**: `pnpm run build` (builds all TypeScript packages)
- **Test entire monorepo**: `pnpm run test` (runs all TypeScript tests)
- **Lint entire monorepo**: `pnpm run lint`

## High-Level Architecture

The Universal Timestamps (UTS) project is a multi-language monorepo implementing a decentralized timestamping protocol that extends OpenTimestamps with Ethereum Attestation Service (EAS) integration.

### Core Components

#### Rust Backend Services (`crates/`)

- **`uts-bmt`**: Binary Merkle Tree implementation with flat-array, power-of-two structure
- **`uts-core`**: Core types, OTS codec, and verification logic
- **`uts-journal`**: RocksDB-backed write-ahead log with at-least-once delivery
- **`uts-calendar`**: HTTP calendar server for digest submission and proof serving
- **`uts-stamper`**: Batching engine that builds Merkle trees and submits EAS attestations
- **`uts-cli`**: Command-line interface for stamping, verifying, and inspecting timestamps
- **`uts-contracts`**: Rust bindings for EAS and L2AnchoringManager contracts
- **`uts-relayer`**: L2→L1→L2 relay service for cross-chain anchoring
- **`uts-beacon-injector`**: Injects drand beacon randomness into timestamping pipeline

#### TypeScript Packages (`packages/`)

- **`@uts/sdk`**: TypeScript/JavaScript SDK for client-side interaction with UTS protocol
- **`@uts/contracts`**: Contract type definitions and ABIs for TypeScript projects

#### Web Application (`apps/web`)

- Vue 3 frontend application with TypeScript
- Uses Pinia for state management and Tailwind CSS for styling
- Integrates with `@uts/sdk` for timestamping functionality

#### Smart Contracts (`contracts/`)

- **L1 contracts**: `L1AnchoringGateway.sol` for anchoring L2 roots on Ethereum
- **L2 contracts**: `L2AnchoringManager.sol` for managing L2 anchoring process
- **Core utilities**: `EASHelper.sol`, `MerkleTree.sol`
- Built on Foundry with EAS and OpenZeppelin integrations

### Dual-Layer Architecture

The system implements two complementary timestamping pipelines:

1. **L2 Direct Path (Fast)**: User digests are batched into Merkle trees and timestamped directly on L2 (Scroll) via EAS attestations, providing low-latency, low-cost timestamps.

2. **L1 Anchoring Path (Secure)**: L2 attestation roots are batched again and anchored on L1 Ethereum through a relayer service, providing high-security guarantees with finality on the main chain.

### Technology Stack

- **Backend**: Rust with Tokio async runtime, Axum web framework, RocksDB storage
- **Blockchain**: Ethereum ecosystem with EAS (Ethereum Attestation Service), Foundry toolchain
- **Frontend**: Vue 3 with TypeScript, Pinia, Tailwind CSS
- **Database**: SQLite with SQLx for database interactions
- **Package Management**: pnpm workspaces for JavaScript/TypeScript monorepo

### Development Environment Requirements

- **Rust**: >= 1.94.0-nightly (e7d44143a 2025-12-24)
- **Cargo**: >= 1.94.0-nightly (3861f60f6 2025-12-19)
- **pnpm**: >= 10.26.2
- **Foundry**: Latest version for smart contract development

### Documentation

Comprehensive documentation is available in the `book/` directory using mdBook format, covering:

- System architecture and component diagrams
- Core primitives and data structures
- Both timestamping pipelines (calendar and L1 anchoring)
- Storage architecture and security considerations
