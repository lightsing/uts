#![feature(thread_sleep_until)]
#![feature(allocator_api)]

//! Calendar server library.

#[macro_use]
extern crate tracing;

use alloy_signer::k256::ecdsa::SigningKey;
use alloy_signer_local::LocalSigner;
use digest::{OutputSizeUser, typenum::Unsigned};
use rocksdb::DB;
use sha3::Keccak256;
use std::sync::Arc;
use uts_journal::Journal;

/// Calendar server routes and handlers.
pub mod routes;
/// Time-related utilities and background tasks.
pub mod time;

/// Application state shared across handlers.
#[derive(Debug)]
pub struct AppState {
    /// Local signer for signing OTS timestamps.
    pub signer: LocalSigner<SigningKey>,
    /// Journal
    pub journal: Journal<{ <Keccak256 as OutputSizeUser>::OutputSize::USIZE }>,
    /// RocksDB
    pub db: Arc<DB>,
}

/// Signal for graceful shutdown.
pub async fn shutdown_signal() {
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
