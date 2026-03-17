use crate::{Sdk, SdkInner};
use alloy_primitives::ChainId;
use alloy_provider::{Provider, ProviderBuilder};
use backon::ExponentialBuilder;
use reqwest::Client;
use std::{
    collections::{BTreeMap, HashSet},
    sync::{Arc, LazyLock},
    time::Duration,
};
use url::Url;
use uts_contracts::provider_helper::{RetryBackoffArgs, ThrottleArgs};

/// Default public calendars to use.
static DEFAULT_CALENDARS: LazyLock<HashSet<Url>> = LazyLock::new(|| {
    HashSet::from([
        Url::parse("https://lgm1.calendar.test.timestamps.now/").unwrap(),
        // Run by Peter Todd
        Url::parse("https://a.pool.opentimestamps.org/").unwrap(),
        Url::parse("https://b.pool.opentimestamps.org/").unwrap(),
        // Run by Riccardo Casatta
        Url::parse("https://a.pool.eternitywall.com/").unwrap(),
        // Run by Bull Bitcoin
        Url::parse("https://ots.btc.catallaxy.com/").unwrap(),
    ])
});

static DEFAULT_PROVIDERS: LazyLock<BTreeMap<ChainId, Url>> = LazyLock::new(|| {
    BTreeMap::from([
        (1, Url::parse("https://0xrpc.io/eth").unwrap()),
        (11155111, Url::parse("https://0xrpc.io/sep").unwrap()),
        (534352, Url::parse("https://rpc.scroll.io").unwrap()),
        (534351, Url::parse("https://sepolia-rpc.scroll.io").unwrap()),
    ])
});

#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    #[error("At least one calendar must be specified")]
    NoCalendars,
    #[error("Quorum of {quorum} is too high for only {calendar_count} calendars")]
    QuorumTooHigh {
        quorum: usize,
        calendar_count: usize,
    },
}

type Result<T, E = BuilderError> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct SdkBuilder {
    http_client: Option<Client>,

    calendars: HashSet<Url>,
    quorum: usize,
    timeout_seconds: u64,
    retry: ExponentialBuilder,

    nonce_size: usize,

    keep_pending: bool,

    eth_providers: BTreeMap<ChainId, Url>,
    eth_compute_units_per_second: u64,
    eth_requests_per_second: u32,

    bitcoin_rpc: Url,
}

impl Default for SdkBuilder {
    fn default() -> Self {
        Self::try_default_from_calendars(DEFAULT_CALENDARS.clone())
            .expect("Default calendars should be valid")
    }
}

impl SdkBuilder {
    /// Create a builder with no calendars and default settings.
    pub fn empty() -> Self {
        Self {
            http_client: None,

            calendars: HashSet::new(),
            quorum: 0,
            timeout_seconds: 5,
            retry: ExponentialBuilder::default(),

            nonce_size: 32,

            keep_pending: false,

            eth_providers: BTreeMap::new(),
            eth_compute_units_per_second: 20,
            eth_requests_per_second: 25,

            bitcoin_rpc: Url::parse("https://bitcoin-rpc.publicnode.com").unwrap(),
        }
    }

    /// Create a builder with the given calendars and default settings.
    pub fn try_default_from_calendars(calendars: impl IntoIterator<Item = Url>) -> Result<Self> {
        let calendars = calendars.into_iter().collect::<HashSet<_>>();
        if calendars.is_empty() {
            return Err(BuilderError::NoCalendars);
        }

        let this = Self {
            calendars,

            eth_providers: DEFAULT_PROVIDERS.clone(),
            ..Self::empty()
        };

        Ok(this.with_two_thirds_quorum())
    }

    /// Set the HTTP client to use for calendar requests.
    ///
    /// If not set, a default client with a user agent will be used.
    pub fn with_http_client(mut self, http_client: Client) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Add a calendar to the builder.
    pub fn add_calendar(mut self, calendar: Url) -> Self {
        self.calendars.insert(calendar);
        self
    }

    /// Set the quorum for the builder. This is capped to at least 1.
    pub fn with_quorum(mut self, quorum: usize) -> Self {
        self.quorum = quorum.max(1);
        self
    }

    /// Set the quorum to 2/3 of the number of calendars, rounded up. This is capped to at least 1.
    pub fn with_two_thirds_quorum(self) -> Self {
        let two_thirds = (self.calendars.len() * 2).div_ceil(3);
        self.with_quorum(two_thirds)
    }

    /// Set the timeout for calendar requests in seconds.
    pub fn with_timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    // Set the retry strategy for calendar requests.

    /// Enable jitter for the backoff.
    pub fn with_jitter(mut self) -> Self {
        self.retry = self.retry.with_jitter();
        self
    }

    /// Set the backoff factor for the backoff.
    pub fn with_backoff_factor(mut self, factor: f32) -> Self {
        self.retry = self.retry.with_factor(factor);
        self
    }

    /// Set the minimum delay for the backoff.
    pub fn with_min_backoff_delay(mut self, min_delay: Duration) -> Self {
        self.retry = self.retry.with_min_delay(min_delay);
        self
    }

    /// Set the maximum delay for the backoff.
    pub fn with_max_backoff_delay(mut self, max_delay: Duration) -> Self {
        self.retry = self.retry.with_max_delay(max_delay);
        self
    }

    /// Set the maximum number of retry attempts for the backoff.
    pub fn with_max_backoff_attempts(mut self, max_times: usize) -> Self {
        self.retry = self.retry.with_max_times(max_times);
        self
    }

    /// Set the maximum total delay for the backoff.
    pub fn with_max_backoff_total_delay(mut self, total_delay: Option<Duration>) -> Self {
        self.retry = self.retry.with_total_delay(total_delay);
        self
    }

    /// Set the size of the nonce to use when stamping digests. If 0, no nonce will be added.
    pub fn with_nonce_size(mut self, nonce_size: usize) -> Self {
        self.nonce_size = nonce_size;
        self
    }

    /// Keep pending attestations in the proof when upgrading.
    pub fn keep_pending(mut self) -> Self {
        self.keep_pending = true;
        self
    }

    /// Add an Ethereum provider for a given chain ID. The URL should point to an Ethereum node that supports the JSON-RPC API.
    pub fn add_eth_provider(mut self, chain_id: ChainId, url: Url) -> Self {
        self.eth_providers.insert(chain_id, url);
        self
    }

    /// Set the compute units per second for Ethereum provider requests. This is used to rate limit requests to avoid overwhelming the provider.
    pub fn with_eth_compute_units_per_second(mut self, compute_units_per_second: u64) -> Self {
        self.eth_compute_units_per_second = compute_units_per_second;
        self
    }

    /// Set the requests per second for Ethereum provider requests. This is used to rate limit requests to avoid overwhelming the provider.
    pub fn with_eth_requests_per_second(mut self, requests_per_second: u32) -> Self {
        self.eth_requests_per_second = requests_per_second;
        self
    }

    /// Build the SDK from the builder, validating the configuration. Returns an error if the configuration is invalid.
    pub fn build(self) -> Result<Sdk> {
        if self.calendars.is_empty() {
            return Err(BuilderError::NoCalendars);
        }
        if self.quorum > self.calendars.len() {
            return Err(BuilderError::QuorumTooHigh {
                quorum: self.quorum,
                calendar_count: self.calendars.len(),
            });
        }

        let http_client = if let Some(client) = self.http_client {
            client
        } else {
            Client::builder()
                .user_agent(concat!("uts/", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("default HTTP client should be valid")
        };

        let eth_retry = RetryBackoffArgs {
            compute_units_per_second: self.eth_compute_units_per_second,
            ..Default::default()
        };
        let eth_throttle = ThrottleArgs {
            requests_per_second: self.eth_requests_per_second,
        };
        let eth_providers = self
            .eth_providers
            .into_iter()
            .map(|(chain_id, url)| {
                let provider = ProviderBuilder::new().connect_client(
                    alloy_rpc_client::ClientBuilder::default()
                        .layer(eth_retry.layer())
                        .layer(eth_throttle.layer())
                        .http(url),
                );
                (chain_id, provider.erased())
            })
            .collect();

        Ok(Sdk {
            inner: Arc::new(SdkInner {
                http_client,

                calendars: self.calendars,
                timeout_seconds: self.timeout_seconds,
                retry: self.retry,
                quorum: self.quorum,

                nonce_size: self.nonce_size,

                keep_pending: self.keep_pending,

                eth_providers,
                bitcoin_rpc: self.bitcoin_rpc,
            }),
        })
    }
}
