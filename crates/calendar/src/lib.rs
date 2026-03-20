//! Calendar server library.

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
