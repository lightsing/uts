# Stamp via CLI

Get your first timestamp in under 5 minutes.

## Prerequisites

- Rust Nightly
- A file to timestamp

## Install the CLI

```bash
cargo install uts-cli --version 0.1.0-alpha.0 --locked
```

Or build from source:

```bash
git clone https://github.com/lightsing/uts.git
cd uts
cargo install --path crates/cli
```

## Timestamp a File

```bash
uts stamp myfile.txt
```

This creates `myfile.txt.ots` containing a pending timestamp proof.

> **Note**:
>
> The pending timestamp cannot prove the existence of the file, and relies
> on the liveness of the calendar server to retrieve the full proof later.

## Upgrade the Timestamp

Wait for the calendar server to attest the batch on-chain (usually ~10 seconds), then run:

```bash
uts upgrade myfile.txt.ots
```

This command retrieves the full self-contained proof.

> **Note**:
>
> Due to the privacy-preserving design of UTS, the calendar server does not
> associate the file hash with the timestamp proof.
>
> Before the [UIP-1](https://github.com/lightsing/uts/issues/59) upgrade, the
> loss of the `.ots` file means permanent loss of the timestamp proof.
>
> Always keep a backup of the `.ots` file if it's important to you.

## Verify the Timestamp

```bash
uts verify myfile.txt
```

The command checks the file with the upgraded proof and confirms if the timestamp is valid.

## Next Steps

- [Architecture](/developer/overview) — How it works
