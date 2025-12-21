//! Calendar server

#[macro_use]
extern crate tracing;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

mod routes;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route(
            "/digest",
            post(routes::ots::submit_digest)
                .layer(DefaultBodyLimit::max(routes::ots::MAX_DIGEST_SIZE)),
        )
        .route(
            "/timestamp/{hex_commitment}",
            get(routes::ots::get_timestamp),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
