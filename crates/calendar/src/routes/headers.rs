use axum::http::HeaderValue;

pub static NO_CACHE: HeaderValue = HeaderValue::from_static("no-cache");
pub static NO_STORE: HeaderValue = HeaderValue::from_static("no-store");
pub static PUBLIC_IMMUTABLE: HeaderValue =
    HeaderValue::from_static("public, max-age=31536000, immutable");

pub static CONTENT_TYPE_JSON: HeaderValue = HeaderValue::from_static("application/json");
pub static CONTENT_TYPE_OCTET_STREAM: HeaderValue =
    HeaderValue::from_static("application/octet-stream");
