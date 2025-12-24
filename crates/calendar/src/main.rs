//! Calendar server

use alloy_primitives::b256;
use alloy_signer_local::LocalSigner;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use std::sync::Arc;
use uts_calendar::{AppState, routes, shutdown_signal, time};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    tokio::spawn(time::async_updater());

    let signer = LocalSigner::from_bytes(&b256!(
        "9ba9926331eb5f4995f1e358f57ba1faab8b005b51928d2fdaea16e69a6ad225"
    ))?;

    let app = Router::new()
        .route(
            "/digest",
            post(routes::ots::submit_digest)
                .layer(DefaultBodyLimit::max(routes::ots::MAX_DIGEST_SIZE)),
        )
        .route(
            "/timestamp/{hex_commitment}",
            get(routes::ots::get_timestamp),
        )
        .with_state(Arc::new(AppState { signer }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
