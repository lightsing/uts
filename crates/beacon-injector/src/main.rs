//! Bot for mocking the behavior of a real user.

// Copyright (C) 2026 UTS Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::config::AppConfig;
use alloy_primitives::{
    Address, B256, Bytes, KECCAK256_EMPTY, Keccak256, U256, keccak256_uncached,
};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_client::ClientBuilder;
use alloy_signer_local::MnemonicBuilder;
use alloy_sol_types::SolValue;
use eyre::ContextCompat;
use reqwest::Client;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    select,
    sync::mpsc::{Receiver, Sender},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument};
use url::Url;
use uts_contracts::{
    eas::{AttestationRequest, AttestationRequestData, EAS, SCHEMA_ID, events::Attested},
    fee_oracle::FeeOracle,
    manager::L2AnchoringManager,
};

mod config;

#[derive(Debug, Deserialize)]
struct Randomness {
    round: u64,
    signature: Bytes,
}

#[derive(Debug, Deserialize)]
struct BeaconInfo {
    period: u64,
}

struct Ctx<P: Provider> {
    config: AppConfig,
    client: Client,
    eas: EAS<P>,
    fee_oracle: FeeOracle<P>,
    l2anchoring_manager: L2AnchoringManager<P>,
    hash_tx: Sender<B256>,
    cancellation_token: CancellationToken,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let config = AppConfig::new()?;

    let cancellation_token = CancellationToken::new();

    let key = MnemonicBuilder::from_phrase(&*config.blockchain.wallet.mnemonic)
        .index(config.blockchain.wallet.index)?
        .build()?;
    let address = key.address();
    info!("Using address: {address}");

    let provider = ProviderBuilder::new()
        .with_simple_nonce_management()
        .wallet(key)
        .connect_client(
            ClientBuilder::default()
                .layer(config.blockchain.rpc.retry.layer())
                .layer(config.blockchain.rpc.throttle.layer())
                .http(config.blockchain.rpc.l2.parse()?),
        );

    let eas = EAS::new(config.blockchain.eas_address, provider.clone());
    let l2anchoring_manager =
        L2AnchoringManager::new(config.blockchain.manager_address, provider.clone());
    let fee_oracle = FeeOracle::new(config.blockchain.fee_oracle_address, provider.clone());

    let client = Client::new();

    let mut beacon_periods: HashMap<String, u64> = client
        .get(config.injector.drand_base_url.join("/v2/beacons")?)
        .header("Accept", "application/json")
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<String>>()
        .await?
        .into_iter()
        .map(|network| (network, 30u64))
        .collect();

    for (network, period) in beacon_periods.iter_mut() {
        let info = client
            .get(
                config
                    .injector
                    .drand_base_url
                    .join(&format!("/v2/beacons/{network}/info"))?,
            )
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?
            .json::<BeaconInfo>()
            .await?;
        *period = info.period;
    }

    let (hash_tx, hash_rx) = tokio::sync::mpsc::channel(1000);
    let ctx = Arc::new(Ctx {
        config,
        client,
        eas,
        fee_oracle,
        l2anchoring_manager,
        hash_tx,
        cancellation_token: cancellation_token.clone(),
    });

    let mut join_set = JoinSet::new();
    for (network, period) in beacon_periods {
        join_set.spawn(ctx.clone().run(network, period));
    }
    join_set.spawn(ctx.clone().run_on_chain(hash_rx));

    select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
            cancellation_token.cancel();
        }
        _ = cancellation_token.cancelled() => { return Ok(()) }
    }

    for task in join_set.join_all().await {
        task?;
    }
    Ok(())
}

impl<P: Provider + 'static> Ctx<P> {
    async fn run(self: Arc<Self>, network: String, period: u64) -> eyre::Result<()> {
        let mut ticker = tokio::time::interval(Duration::from_secs(period));
        let mut round = 0;
        let drand_url = self
            .config
            .injector
            .drand_base_url
            .join(&format!("/v2/beacons/{network}/rounds/latest"))?;
        loop {
            if let Err(e) = self.clone().run_inner(drand_url.clone(), &mut round).await {
                error!(%network, %round, "{e:?}");
            }
            select! {
                _ = ticker.tick() => {}
                _ = self.cancellation_token.cancelled() => { return Ok(()) }
            }
        }
    }

    async fn run_inner(self: Arc<Self>, drand_url: Url, round: &mut u64) -> eyre::Result<()> {
        let randomness = self
            .client
            .get(drand_url)
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?
            .json::<Randomness>()
            .await?;
        if *round == randomness.round {
            return Ok(());
        }
        *round = randomness.round;

        let hash = keccak256_uncached(&*randomness.signature);
        // trace!(%randomness.round, %randomness.signature, %hash);

        tokio::spawn(self.clone().request_calendar(hash));
        self.hash_tx.send(hash).await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn request_calendar(self: Arc<Self>, hash: B256) -> eyre::Result<()> {
        let calendar_url = self.config.injector.calendar_url.join("digest")?;
        self.client
            .post(calendar_url)
            .body(hash.to_vec())
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        Ok(())
    }

    async fn run_on_chain(self: Arc<Self>, mut hash_rx: Receiver<B256>) -> eyre::Result<()> {
        loop {
            let timeout = tokio::time::sleep(Duration::from_secs(5));
            let mut hasher = Keccak256::new();
            tokio::pin!(timeout);
            loop {
                select! {
                    Some(hash) = hash_rx.recv() => {
                        info!(%hash, "Received hash from calendar request");
                        hasher.update(hash);
                    }
                    _ = &mut timeout => {
                        info!("Hash collection timeout reached, finalizing hash and submitting on-chain if not empty");
                        break
                    }
                    _ = self.cancellation_token.cancelled() => { return Ok(()) }
                }
            }
            let hash = hasher.finalize();
            info!(%hash, "Finalized hash after collection period");
            if hash == KECCAK256_EMPTY {
                continue;
            }
            if let Err(e) = self.submit_on_chain(hash).await {
                error!(%hash, "{e:?}");
            }
        }
    }

    #[instrument(skip(self), err)]
    async fn submit_on_chain(&self, hash: B256) -> eyre::Result<()> {
        let receipt = self
            .eas
            .attest(AttestationRequest {
                schema: SCHEMA_ID,
                data: AttestationRequestData {
                    recipient: Address::ZERO,
                    expirationTime: 0,
                    revocable: false,
                    refUID: B256::ZERO,
                    data: hash.abi_encode().into(),
                    value: U256::ZERO,
                },
            })
            .send()
            .await?
            .get_receipt()
            .await?;
        let attested = receipt
            .decoded_log::<Attested>()
            .context("Attested event not found")?;

        let floor_fee = self.fee_oracle.getFloorFee().call().await?;
        let fee = floor_fee * U256::from(110) / U256::from(100); // add 10% buffer
        self.l2anchoring_manager
            .submitForL1Anchoring(attested.uid)
            .value(fee)
            .send()
            .await?
            .watch()
            .await?;
        info!(%hash, uid = %attested.uid, "Attestation submitted on-chain");
        Ok(())
    }
}
