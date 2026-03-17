//! Rust SDK for the Universal Timestamps protocol.

use backon::{ExponentialBuilder, Retryable};
use bytes::Bytes;
use http::StatusCode;
use reqwest::{Client, RequestBuilder};
use std::{collections::HashSet, sync::Arc, time::Duration};
use tracing::trace;
use url::Url;
#[cfg(feature = "eas-verifier")]
use {alloy_primitives::ChainId, alloy_provider::DynProvider, std::collections::BTreeMap};

mod builder;
mod error;
mod stamp;
mod upgrade;
mod verify;

pub use error::Error;
pub use upgrade::UpgradeResult;

/// Alias `Result` to use the crate's error type by default.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// SDK for interacting with Universal Timestamping protocol.
#[derive(Debug, Clone)]
pub struct Sdk {
    inner: Arc<SdkInner>,
}

#[derive(Debug)]
struct SdkInner {
    http_client: Client,

    // Stamp
    calendars: HashSet<Url>,
    quorum: usize,
    timeout_seconds: u64,
    retry: ExponentialBuilder,

    // Privacy
    nonce_size: usize,

    // Upgrade
    keep_pending: bool,

    // Verify
    #[cfg(feature = "eas-verifier")]
    eth_providers: BTreeMap<ChainId, DynProvider>,
    #[cfg(feature = "bitcoin-verifier")]
    bitcoin_rpc: Url,
}

impl Default for Sdk {
    fn default() -> Self {
        Self::new()
    }
}

impl Sdk {
    /// Create a new SDK with default settings.
    pub fn new() -> Self {
        Self::builder()
            .build()
            .expect("Default SDK should be valid")
    }

    /// Create a new SDK builder with default settings.
    pub fn builder() -> builder::SdkBuilder {
        builder::SdkBuilder::default()
    }

    /// Create a new SDK builder with given calendars and default settings.
    pub fn try_builder_from_calendars(
        calendars: impl IntoIterator<Item = Url>,
    ) -> Result<builder::SdkBuilder, builder::BuilderError> {
        builder::SdkBuilder::try_default_from_calendars(calendars)
    }

    async fn http_request_with_retry<Builder>(
        &self,
        method: http::Method,
        url: Url,
        response_size_limit: usize,
        builder_fn: Builder,
    ) -> Result<(http::response::Parts, Bytes)>
    where
        Builder: Fn(RequestBuilder) -> RequestBuilder + Send + Sync + 'static,
    {
        let client = self.inner.http_client.clone();
        let timeout_seconds = self.inner.timeout_seconds;
        let res = {
            move || {
                let client = client.clone();
                let method = method.clone();
                let url = url.clone();
                let req = client
                    .request(method, url)
                    .timeout(Duration::from_secs(timeout_seconds));
                let req = builder_fn(req);

                async move {
                    let res = req.send().await?;
                    if res.status().is_server_error()
                        || (
                            // specially treat 404 as non-error
                            res.status().is_client_error() && res.status() != StatusCode::NOT_FOUND
                        )
                    {
                        res.error_for_status()
                    } else {
                        Ok::<_, reqwest::Error>(res)
                    }
                }
            }
        }
        .retry(self.inner.retry)
        .when(|e| {
            if e.is_connect() || e.is_timeout() {
                return true;
            }
            if let Some(status) = e.status() {
                return status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS;
            }
            false
        })
        .notify(|e, duration| {
            trace!("retrying error {e:?} after sleeping {duration:?}");
        })
        .await?;

        let res: http::Response<reqwest::Body> = res.into();
        let (parts, body) = res.into_parts();
        let body = http_body_util::Limited::new(body, response_size_limit);
        let bytes = http_body_util::BodyExt::collect(body)
            .await
            .map_err(Error::Http)?
            .to_bytes();
        Ok((parts, bytes))
    }
}
