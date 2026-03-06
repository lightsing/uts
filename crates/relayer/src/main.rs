//! The relayer for collecting L2 events and submitting them to L1.

use alloy_provider::{ProviderBuilder, WsConnect};
use alloy_rpc_client::ClientBuilder;
use sqlx::{
    migrate,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::fs;
use tokio_util::sync::CancellationToken;
use uts_contracts::{gateway::L1AnchoringGateway, manager::L2AnchoringManager};
use uts_relayer::{config::AppConfig, indexer::L2Indexer, relayer::Relayer};

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

    let l1_provider = ProviderBuilder::new().connect_client(
        ClientBuilder::default()
            .layer(config.blockchain.rpc.retry.layer())
            .layer(config.blockchain.rpc.throttle.layer())
            .http(config.blockchain.rpc.l1.parse()?),
    );
    let l2_provider = ProviderBuilder::new().connect_client(
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
        db,
        gateway,
        manager,
        config.relayer,
        cancellation_token.clone(),
    )
    .await?;

    // spawn the subscriber tasks first.
    tokio::spawn(l2_indexer.clone().start_subscribers());

    // block waiting for catch up, ensure data is up to date before trying to submit any transactions to L1.
    l2_indexer.start_scanners().await?;

    relayer.run().await?;

    Ok(())
}
