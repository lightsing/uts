# UTS Cli Tool

## Quick Start

### 1. Create a Timestamp (`stamp`)

Submit files to remote calendar servers for initial attestation:

```bash
# Timestamp one or more files
uts stamp file1.txt file2.zip

# Specify custom calendars and a quorum requirement
uts stamp -c https://calendar.example.com -m 1 document.pdf

# Use a specific hashing algorithm
uts stamp --hasher sha256 photo.jpg

```

This will create a corresponding `.ots` proof file (e.g., `document.pdf.ots`).

### 2. Upgrade a Proof (`upgrade`)

Initial proofs are often "pending." Once the calendar server commits the Merkle root to a blockchain, you must upgrade the proof to include the full path to the block:

```bash
uts upgrade document.pdf.ots

```

### 3. Verify a Proof (`verify`)

Verify that a file matches its timestamp proof and check the attestation status on the calendar or blockchain:

```bash
# Automatically finds the matching .ots file
uts verify document.pdf

# Specify an Ethereum RPC provider for on-chain verification
uts verify document.pdf --eth-provider https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY

```

### 4. Inspect a Proof (`inspect`)

View the internal structure, opcodes, and attestation paths of an `.ots` file in a human-readable format:

```bash
uts inspect document.pdf.ots

```

## Command Reference

### `stamp`

| Argument | Description | Default |
| --- | --- | --- |
| `-c, --calendar` | Remote calendar URL (can be specified multiple times) | Built-in list |
| `-m` | Minimum quorum of calendars required | 1 |
| `-H, --hasher` | Hashing algorithm (`keccak256`, `sha256`, `sha1`, `ripemd160`) | `keccak256` |
| `--timeout` | Timeout in seconds for calendar responses | 5 |

### `verify`

| Argument | Description |
| --- | --- |
| `file` | The target file to verify |
| `stamp_file` | (Optional) Explicit path to the `.ots` file |
| `--eth-provider` | (Optional) Ethereum RPC URL for UTS contract verification |


