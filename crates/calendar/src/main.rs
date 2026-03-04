//! Calendar server

use alloy_primitives::b256;
use alloy_provider::{Provider, ProviderBuilder, network::EthereumWallet};
use alloy_signer_local::{LocalSigner, MnemonicBuilder};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::Method,
    routing::{get, post},
};
use digest::{OutputSizeUser, typenum::Unsigned};
use eyre::ContextCompat;
use rocksdb::DB;
use sha3::Keccak256;
use std::{env, path::PathBuf, sync::Arc};
use tower_http::{cors, cors::CorsLayer};
use tracing::info;
use uts_calendar::{AppState, routes, shutdown_signal, time};
use uts_contracts::eas::{EAS, EAS_ADDRESSES};
use uts_journal::{Journal, JournalConfig, checkpoint::CheckpointConfig};
use uts_stamper::{Stamper, StamperConfig};

const RING_BUFFER_CAPACITY: usize = 1 << 20; // 1 million entries

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    tokio::spawn(time::async_updater());

    let signer = LocalSigner::from_bytes(&b256!(
        "9ba9926331eb5f4995f1e358f57ba1faab8b005b51928d2fdaea16e69a6ad225"
    ))?;

    // journal
    let journal = Journal::with_capacity_and_config(
        RING_BUFFER_CAPACITY,
        JournalConfig {
            consumer_checkpoint: CheckpointConfig {
                path: PathBuf::from("./.journal/.checkpoint"),
                ..Default::default()
            },
            wal_dir: PathBuf::from("./.journal"),
        },
    )?;

    let key = MnemonicBuilder::from_phrase(env::var("MNEMONIC")?.as_str())
        .index(0u32)?
        .build()?;
    info!("Using address: {:?}", key.address());
    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::new(key))
        .connect("https://sepolia-rpc.scroll.io")
        .await?;
    let chain_id = provider.get_chain_id().await?; // sanity check

    let eas_address = *EAS_ADDRESSES
        .get(&chain_id)
        .context("eas default address not found")?;
    let contract = EAS::new(eas_address, provider.clone());

    // stamper
    let reader = journal.reader();
    let db = Arc::new(DB::open_default("./.db/tries")?);
    let mut stamper =
        Stamper::<Keccak256, _, { <Keccak256 as OutputSizeUser>::OutputSize::USIZE }>::new(
            reader,
            db.clone(),
            contract,
            // TODO: tune configuration
            StamperConfig {
                max_interval_seconds: 10,
                max_entries_per_timestamp: 1 << 10, // 1024 entries
                min_leaves: 1 << 4,
                max_cache_size: 256,
            },
        );
    // TODO: graceful shutdown
    tokio::spawn(async move {
        stamper.run().await;
    });

    let app = Router::new()
        .route(
            "/digest",
            post(routes::ots::submit_digest)
                .layer(DefaultBodyLimit::max(routes::ots::MAX_DIGEST_SIZE)),
        )
        .route("/timestamp/{commitment}", get(routes::ots::get_timestamp))
        .with_state(Arc::new(AppState {
            signer,
            journal: journal.clone(),
            db,
        }))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(cors::Any),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // this will join the journal's background task and ensure flush of all pending commits
    journal.shutdown()?;

    Ok(())
}
