//! The relayer for collecting L2 events and submitting them to L1.

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

use alloy_primitives::bytes::Bytes;
use alloy_provider::{ProviderBuilder, WsConnect};
use alloy_rpc_client::ClientBuilder;
use alloy_signer_local::MnemonicBuilder;
use axum::{Router, http::StatusCode, response::Html, routing::get};
use sqlx::{
    migrate,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{fs, sync::Arc};
use tokio_util::sync::CancellationToken;
use tracing::info;
use uts_contracts::{gateway::L1AnchoringGateway, manager::L2AnchoringManager};
use uts_relayer::{
    AppState, config::AppConfig, indexer::L2Indexer, relayer::Relayer, shutdown_signal,
};

const INDEX_PAGE_TEMPLATE: &str = include_str!("index.html");

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    // FIXME: some crate enables `rustls/ring` cause `rustls` don't know which provider to use and thus requires the user to explicitly set the provider.
    #[cfg(feature = "reqwest-rustls-tls")]
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();

    let config = AppConfig::new()?;

    let cancellation_token = CancellationToken::new();

    fs::create_dir_all(config.db.sql.filename.parent().unwrap())?;
    let db = SqlitePoolOptions::new()
        .connect_with(
            SqliteConnectOptions::new()
                .filename(config.db.sql.filename)
                .create_if_missing(true)
                .foreign_keys(true),
        )
        .await?;
    migrate!().run(&db).await?;

    let key = MnemonicBuilder::from_phrase(&*config.blockchain.wallet.mnemonic)
        .index(config.blockchain.wallet.index)?
        .build()?;
    let address = key.address();
    info!("Using address: {address}");

    let l1_provider = ProviderBuilder::new()
        .with_simple_nonce_management()
        .wallet(key.clone())
        .connect_client(
            ClientBuilder::default()
                .layer(config.blockchain.rpc.retry.layer())
                .layer(config.blockchain.rpc.throttle.layer())
                .http(config.blockchain.rpc.l1.parse()?),
        );
    let l2_provider = ProviderBuilder::new()
        .with_simple_nonce_management()
        .wallet(key.clone())
        .connect_client(
            ClientBuilder::default()
                .layer(config.blockchain.rpc.retry.layer())
                .layer(config.blockchain.rpc.throttle.layer())
                .ws(WsConnect::new(&*config.blockchain.rpc.l2_ws))
                .await?,
        );

    let gateway = L1AnchoringGateway::new(config.blockchain.gateway_address, l1_provider);
    let manager = L2AnchoringManager::new(config.blockchain.manager_address, l2_provider);

    let l2_indexer = L2Indexer::new(
        db.clone(),
        manager.clone(),
        config.indexer.l2,
        cancellation_token.clone(),
    )
    .await?;

    let relayer = Relayer::new(
        db.clone(),
        key.address(),
        gateway,
        manager,
        config.relayer,
        cancellation_token.clone(),
    )
    .await?;

    let html = INDEX_PAGE_TEMPLATE
        .replace("{{VERSION}}", env!("CARGO_PKG_VERSION"))
        .replace("{{NODE_NAME}}", config.server.node_name.as_str())
        .replace("{{NODE_ADDRESS}}", &address.to_string());
    let html = Bytes::from(html);

    let app = Router::new()
        .route("/", get(|| async move { Html(html.clone()) }))
        .route("/healthcheck", get(|| async { StatusCode::NO_CONTENT }))
        .route("/metrics", get(uts_relayer::metrics))
        .with_state(Arc::new(AppState { db }));

    let listener = tokio::net::TcpListener::bind(&*config.server.bind_address).await?;

    tokio::spawn(
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal(cancellation_token.clone()))
            .into_future(),
    );

    // spawn the subscriber tasks first.
    tokio::spawn(l2_indexer.clone().start_subscribers());

    // block waiting for catch up, ensure data is up to date before trying to submit any transactions to L1.
    l2_indexer.start_scanners().await?;

    info!("L2 indexer is up to date, starting relayer");

    relayer.run().await?;

    Ok(())
}
