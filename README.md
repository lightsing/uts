# Universal Timestamps (UTS)

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![CI (Rust)](https://github.com/lightsing/uts/actions/workflows/ci-rust.yml/badge.svg)](https://github.com/lightsing/uts/actions/workflows/ci-rust.yml)
[![CI (TypeScript)](https://github.com/lightsing/uts/actions/workflows/ci-typescript.yml/badge.svg)](https://github.com/lightsing/uts/actions/workflows/ci-typescript.yml)

Universal Timestamps is a super set of [opentimestamps](https://opentimestamps.org/).

UTS batches user-submitted digests into Merkle trees and anchors the roots on-chain
via [EAS](https://attest.org/) attestations, providing trustless, verifiable
timestamps without relying on a single trusted calendar server.

## Quick Start

```bash
cargo install uts-cli --version 0.1.0-alpha.1 --locked
```

## Links

- **Book**: <https://book.timestamps.now/>
- **Calendar**: <https://lgm1.calendar.test.timestamps.now/>
- **Relayer**: <https://lich.relayer.test.timestamps.now/>

## Development

### Pre-requisites

- Rust Toolchain == 1.94.0
- pnpm >= 10.26.2

See [CONTRIBUTING.md](CONTRIBUTING.md) for full development setup and guidelines.

## Supporting the Project

You can support the UTS project by sending ETH directly to the operator address
displayed on the [calendar](https://lgm1.calendar.test.timestamps.now/) or
[relayer](https://lich.relayer.test.timestamps.now/) home page.

## About Our Codenames: The Universe's Natural Clocks

When choosing a codename theme, pulsars serve as the perfect physical metaphor.

Often referred to as the "lighthouses of the universe" and "natural atomic clocks,"
pulsars rotate at high speeds with incredibly stable and precise periods, emitting
regular pulses of electromagnetic radiation. Their timekeeping precision can even
rival that of humanity's most advanced atomic clocks.

Adopting astronomically significant pulsars like LGM-1, Vela, or Swift as codenames
for our various environments or microservices does more than just inject a touch of
hardcore, geeky romance into the codebase. It represents our highest engineering
aspiration: an architecture that runs as eternally and precisely as the stars
themselves.

Allocation Tracker: https://github.com/lightsing/uts/issues/46

## License

This project uses a split licensing model:

- **Core Libraries** are dual-licensed under [MIT](LICENSE-MIT) or
  [Apache-2.0](LICENSE-APACHE), at your option.
- **All SDKs** are dual-licensed under [MIT](LICENSE-MIT) or
  [Apache-2.0](LICENSE-APACHE), at your option.
- **Server components** are licensed under [AGPL-3.0](LICENSE-AGPL).

See the individual directory for per-package license declarations.
