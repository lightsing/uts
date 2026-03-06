use alloy_transport::layers::{RetryBackoffLayer, ThrottleLayer};

/// Parameters for retrying rate-limited requests with exponential backoff.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "provider-helper-clap", derive(clap::Args))]
#[cfg_attr(
    feature = "provider-helper-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct RetryBackoffArgs {
    /// Max number of retries for rate-limited requests.
    #[cfg_attr(
        feature = "provider-helper-clap",
        arg(long, help = "Maximum number of retries", default_value = "10")
    )]
    pub max_rate_limit_retries: u32,
    /// Initial backoff in milliseconds for retrying rate-limited requests.
    #[cfg_attr(
        feature = "provider-helper-clap",
        arg(long, help = "Initial backoff in milliseconds", default_value = "100")
    )]
    pub initial_backoff: u64,
    /// Compute units per second for rate-limiting purposes.
    #[cfg_attr(
        feature = "provider-helper-clap",
        arg(long, help = "Compute units per second", default_value = "100")
    )]
    pub compute_units_per_second: u64,
}

/// Parameters for throttling requests to the RPC provider.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "provider-helper-clap", derive(clap::Args))]
#[cfg_attr(
    feature = "provider-helper-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct ThrottleArgs {
    /// Requests per second to throttle.
    #[cfg_attr(
        feature = "provider-helper-clap",
        arg(long, help = "Requests per second to throttle", default_value = "5")
    )]
    pub requests_per_second: u32,
}

impl Default for RetryBackoffArgs {
    fn default() -> Self {
        Self {
            max_rate_limit_retries: 10,
            initial_backoff: 100,
            compute_units_per_second: 20,
        }
    }
}

impl Default for ThrottleArgs {
    fn default() -> Self {
        Self {
            requests_per_second: 25,
        }
    }
}

impl RetryBackoffArgs {
    /// Get a `RetryBackoffLayer` from the retry backoff arguments.
    pub fn layer(&self) -> RetryBackoffLayer {
        RetryBackoffLayer::new(
            self.max_rate_limit_retries,
            self.initial_backoff,
            self.compute_units_per_second,
        )
    }
}

impl ThrottleArgs {
    /// Get a `RetryBackoffLayer` from the retry backoff arguments.
    pub fn layer(&self) -> ThrottleLayer {
        ThrottleLayer::new(self.requests_per_second)
    }
}
