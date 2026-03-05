use crate::routes::headers::{CONTENT_TYPE_JSON, NO_CACHE, NO_STORE};
use axum::{
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};

#[inline]
pub fn service_unavailable() -> Response {
    let mut headers = HeaderMap::with_capacity(2);
    headers.insert(header::CACHE_CONTROL, NO_CACHE.clone());
    headers.insert(header::CONTENT_TYPE, CONTENT_TYPE_JSON.clone());

    (
        StatusCode::SERVICE_UNAVAILABLE,
        headers,
        r#"{"err":"server busy"}"#,
    )
        .into_response()
}

#[inline]
pub fn not_found() -> Response {
    let mut headers = HeaderMap::with_capacity(2);
    headers.insert(header::CACHE_CONTROL, NO_CACHE.clone());
    headers.insert(header::CONTENT_TYPE, CONTENT_TYPE_JSON.clone());

    (StatusCode::NOT_FOUND, headers, r#"{"err":"not found"}"#).into_response()
}

#[inline]
pub fn internal_server_error() -> Response {
    let mut headers = HeaderMap::with_capacity(2);
    headers.insert(header::CACHE_CONTROL, NO_STORE.clone());
    headers.insert(header::CONTENT_TYPE, CONTENT_TYPE_JSON.clone());

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        headers,
        r#"{"err":"internal error"}"#,
    )
        .into_response()
}
