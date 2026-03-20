//! Relayer for L1 anchoring.

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

use crate::sql::{l1_batch::L1Batch, stats::CulminationCosts};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

uts_sql::migrator!("./migrations");

/// Indexer.
pub mod indexer;

/// Relayer.
pub mod relayer;

pub(crate) mod sql;

/// Configuration.
pub mod config;

/// Application state shared across handlers.
#[derive(Debug)]
pub struct AppState {
    /// Sqlite pool
    pub db: sqlx::SqlitePool,
}

#[derive(Debug, Serialize)]
struct Metrics {
    total: i64,
    pending: i64,
    latest_batch: Option<L1Batch>,
    costs: CulminationCosts,
}

fn internal_server_error() -> Response {
    let mut headers = HeaderMap::with_capacity(1);
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        headers,
        r#"{"error": "Internal Server Error"}"#,
    )
        .into_response()
}

/// Handler for the `/metrics` endpoint, which returns JSON-formatted metrics about the calendar server.
pub async fn metrics(State(state): State<Arc<AppState>>) -> Response {
    let mut headers = HeaderMap::with_capacity(2);
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    // cache for 10s
    headers.insert(
        axum::http::header::CACHE_CONTROL,
        "public, max-age=10".parse().unwrap(),
    );

    let Ok(total) = sql::l1_batch::count_l1batches(&state.db).await else {
        return internal_server_error();
    };
    let Ok(latest_batch) = sql::l1_batch::get_latest_l1batch(&state.db).await else {
        return internal_server_error();
    };
    let Ok(pending) = sql::anchoring_requests::count_pending_events(
        &state.db,
        latest_batch.map(|b| b.start_index + b.count).unwrap_or(1),
    )
    .await
    else {
        return internal_server_error();
    };
    let Ok(costs) = sql::stats::get_culmination_costs(&state.db).await else {
        return internal_server_error();
    };

    let metrics = Metrics {
        total,
        pending,
        latest_batch,
        costs,
    };

    (StatusCode::OK, headers, Json(metrics)).into_response()
}

/// Signal for graceful shutdown.
pub async fn shutdown_signal(cancellation_token: CancellationToken) {
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
        _ = cancellation_token.cancelled() => {},
    };

    cancellation_token.cancel();
}
