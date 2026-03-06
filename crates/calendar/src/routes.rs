use crate::{
    AppState,
    routes::{headers::CONTENT_TYPE_JSON, responses::internal_server_error},
};
use alloy_primitives::private::serde::Serialize;
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use uts_stamper::sql;

/// ots related routes
pub mod ots;

mod headers;
mod responses;

#[derive(Debug, Serialize)]
struct Metrics {
    total: u64,
    pending: u64,
    stamper: sql::Stats,
}

/// Handler for the `/metrics` endpoint, which returns JSON-formatted metrics about the calendar server.
pub async fn metrics(State(state): State<Arc<AppState>>) -> Response {
    let mut headers = HeaderMap::with_capacity(2);
    headers.insert(CONTENT_TYPE, CONTENT_TYPE_JSON.clone());
    // cache for 1 minute
    headers.insert(
        axum::http::header::CACHE_CONTROL,
        "public, max-age=10".parse().unwrap(),
    );

    let Ok(stamper) = sql::get_stats(&state.sql_pool).await else {
        return internal_server_error();
    };

    let total = state.journal.write_index();
    let pending = total - state.journal.consumed_index();

    let metrics = Metrics {
        total,
        pending,
        stamper,
    };

    (StatusCode::OK, headers, Json(metrics)).into_response()
}
