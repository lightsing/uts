//! Calendar server

use alloy_provider::{Provider, ProviderBuilder, network::EthereumWallet};
use alloy_rpc_client::ClientBuilder;
use alloy_signer_local::MnemonicBuilder;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{Method, StatusCode},
    response::Html,
    routing::{get, post},
};
use bytes::Bytes;
use eyre::{Context, ContextCompat};
use rocksdb::DB;
use sha3::Keccak256;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{convert::Infallible, env, fs, sync::Arc, time::Duration};
use tokio_util::sync::CancellationToken;
use tower_http::{cors, cors::CorsLayer};
use tracing::{error, info};
use uts_calendar::{AppState, config::AppConfig, routes, shutdown_signal, time};
use uts_contracts::eas::{EAS, EAS_ADDRESSES};
use uts_journal::{Journal, JournalConfig};
use uts_stamper::{Stamper, StamperConfig, sql};

const RING_BUFFER_CAPACITY: usize = 1 << 20; // 1 million entries
const INDEX_PAGE_TEMPLATE: &str = include_str!("index.html");

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let config = AppConfig::new()?;

    tokio::spawn(time::async_updater());

    let token = CancellationToken::new();

    // journal
    let journal = Journal::with_capacity_and_config(
        RING_BUFFER_CAPACITY,
        JournalConfig {
            db_path: config.db.journal.db_path.clone(),
        },
    )?;

    let key = MnemonicBuilder::from_phrase(&*config.blockchain.wallet.mnemonic)
        .index(config.blockchain.wallet.index)?
        .build()?;
    let address = key.address();
    info!("Using address: {:?}", key.address());

    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::new(key.clone()))
        .connect_client(
            ClientBuilder::default()
                .layer(config.blockchain.rpc.retry.layer())
                .layer(config.blockchain.rpc.throttle.layer())
                .http(config.blockchain.rpc.url.parse()?),
        );
    let chain_id = provider.get_chain_id().await?; // sanity check
    let eas_address = *EAS_ADDRESSES
        .get(&chain_id)
        .context("eas default address not found")?;
    let contract = EAS::new(eas_address, provider.clone());

    // stamper
    let reader = journal.reader();
    fs::create_dir_all(config.db.kv.path.parent().unwrap())?;
    let db = Arc::new(DB::open_default(&config.db.kv.path)?);
    fs::create_dir_all(config.db.sql.filename.parent().unwrap())?;
    let sql = SqlitePoolOptions::new()
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&config.db.sql.filename)
                .create_if_missing(true)
                .foreign_keys(true),
        )
        .await?;
    sql::migrate(&sql)
        .await
        .context("failed to run database migrations")?;

    let mut stamper = Stamper::<Keccak256, _>::new(
        reader,
        db.clone(),
        sql.clone(),
        contract,
        StamperConfig {
            max_interval_seconds: config.stamper.max_interval_seconds,
            max_entries_per_timestamp: config.stamper.max_entries_per_timestamp,
            min_leaves: config.stamper.min_leaves,
        },
    );

    {
        let token = token.clone();
        tokio::spawn(async move {
            if let Err(e) = stamper.run(token.clone()).await {
                error!("stamper fatal error: {e}");
                token.cancel();
            }
        });
    }

    let listener = tokio::net::TcpListener::bind(&*config.server.bind_address).await?;

    // compatible API.
    let public_api = Router::new()
        .route(
            "/digest",
            post(routes::ots::submit_digest)
                .layer::<_, Infallible>(
                    CorsLayer::new()
                        .allow_methods([Method::POST])
                        .allow_origin(cors::Any)
                        .max_age(Duration::from_hours(24)),
                )
                .layer::<_, Infallible>(DefaultBodyLimit::max(routes::ots::MAX_DIGEST_SIZE)),
        )
        .route(
            "/timestamp/{commitment}",
            get(routes::ots::get_timestamp).layer::<_, Infallible>(
                CorsLayer::new()
                    .allow_methods([Method::GET])
                    .allow_origin(cors::Any)
                    .max_age(Duration::from_hours(24)),
            ),
        );

    let html = INDEX_PAGE_TEMPLATE
        .replace("{{VERSION}}", env!("CARGO_PKG_VERSION"))
        .replace("{{NODE_NAME}}", config.server.node_name.as_str())
        .replace("{{NODE_ADDRESS}}", &address.to_string());
    let html = Bytes::from(html);

    let app = Router::new()
        .route("/", get(|| async move { Html(html.clone()) }))
        .route("/healthcheck", get(|| async { StatusCode::NO_CONTENT }))
        .route("/metrics", get(routes::metrics))
        .merge(public_api)
        .with_state(Arc::new(AppState {
            config,
            signer: key.clone(),
            journal: journal.clone(),
            kv_db: db,
            sql_pool: sql,
        }));

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(async move {
            token.cancelled().await;
            error!("fatal error signal received, shutting down");
        }))
        .await?;

    Ok(())
}
