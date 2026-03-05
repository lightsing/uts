#![feature(thread_sleep_until)]
#![feature(allocator_api)]

//! Calendar server library.

#[macro_use]
extern crate tracing;

use crate::config::AppConfig;
use alloy_signer::k256::ecdsa::SigningKey;
use alloy_signer_local::LocalSigner;
use rocksdb::DB;
use std::sync::Arc;
use uts_journal::Journal;

/// Config
pub mod config;
/// Calendar server routes and handlers.
pub mod routes;
/// Time-related utilities and background tasks.
pub mod time;

/// Application state shared across handlers.
#[derive(Debug)]
pub struct AppState {
    /// Application configuration.
    pub config: AppConfig,
    /// Local signer for signing OTS timestamps.
    pub signer: LocalSigner<SigningKey>,
    /// Journal
    pub journal: Journal,
    /// RocksDB
    pub kv_db: Arc<DB>,
    /// Sqlite pool
    pub sql_pool: sqlx::SqlitePool,
}

/// Signal for graceful shutdown.
pub async fn shutdown_signal(fatal_error_happens: impl Future<Output = ()>) {
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
        _ = fatal_error_happens => {},
    }
}
