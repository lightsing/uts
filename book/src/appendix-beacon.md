# Appendix A: Beacon Injector

The beacon injector (`uts-beacon-injector`) is an auxiliary service that injects [drand](https://drand.love/) randomness beacons into the UTS timestamping pipeline, providing a continuous stream of publicly verifiable, unpredictable timestamps.

## What is drand?

[drand](https://drand.love/) (distributed randomness) is a decentralized randomness beacon that produces publicly verifiable, unbiased, and unpredictable random values at regular intervals. A network of independent nodes runs a distributed key generation protocol and produces BLS threshold signatures on sequential round numbers.

Each beacon round produces:
- A **round number** (monotonically increasing).
- A **BLS signature** over the round number (the randomness).

The signature is deterministic for a given round — once the round is produced, anyone can verify it using the beacon's public key.

## Why Inject Randomness?

Injecting drand beacons into UTS serves two purposes:

1. **Liveness proof**: A continuous stream of timestamps proves the system is operational. If beacon timestamps stop appearing, it signals a service disruption.
2. **Unpredictable anchoring**: Since drand outputs are unpredictable before they are produced, timestamping them proves the system was operational *at that specific moment* — the timestamp could not have been pre-computed.

## Beacon Periods and Rounds

The injector discovers available drand networks and their periods:

```
GET {drand_base_url}/v2/beacons              → list of networks
GET {drand_base_url}/v2/beacons/{net}/info    → { period: u64 }
```

For each network, a separate task polls for new rounds at the network's period interval:

```
GET {drand_base_url}/v2/beacons/{net}/rounds/latest  → { round, signature }
```

If the round number hasn't changed since the last poll, the iteration is skipped.

## How It Submits Attestations

For each new drand round, the injector:

### 1. Hash the Beacon Signature

```rust
let hash = keccak256(&randomness.signature);
```

### 2. Submit to Calendar Server

The hash is posted to the calendar server's `/digest` endpoint, entering the normal calendar timestamping pipeline:

```rust
// Async: submit to calendar
request_calendar(hash).await;
```

### 3. Submit for L1 Anchoring

In parallel, hashes are collected over a 5-second window and batched:

```rust
// Collect hashes for 5 seconds
let batch_hash = keccak256(collected_hashes);

// Create EAS attestation
let uid = eas.attest(batch_hash).send().await?;

// Get fee with 10% buffer
let fee = fee_oracle.getFloorFee() * 110 / 100;

// Submit for L1 anchoring
l2_manager.submitForL1Anchoring(uid).value(fee).send().await?;
```

This dual submission ensures the beacon data is timestamped via both:
- **Pipeline A**: Calendar timestamping (fast, L2-only).
- **Pipeline B**: L1 anchoring (slower, L1 finality).

## Multi-Chain Deployment

The injector supports multiple drand networks simultaneously. Each network runs its own polling task, and all hashes flow into the same calendar and L1 anchoring pipeline.

## Configuration

```rust
pub struct AppConfig {
    pub blockchain: BlockchainConfig {
        pub eas_address: Address,
        pub manager_address: Address,
        pub fee_oracle_address: Address,
        pub rpc: RpcConfig { l2: String, ... },
        pub wallet: WalletConfig { mnemonic: String, index: u32 },
    },
    pub injector: InjectorConfig {
        pub drand_base_url: Url,
        pub calendar_url: Url,
    },
}
```

The service connects to:
- The **drand HTTP API** for beacon data.
- The **calendar server** for L2 timestamping.
- The **L2 blockchain** (via RPC) for EAS attestations and L1 anchoring submissions.
