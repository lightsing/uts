use alloy_primitives::Address;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use url::Url;
use uts_contracts::provider_helper::{RetryBackoffArgs, ThrottleArgs};

/// Application configuration loaded from defaults, a config file, and environment variables.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    /// Blockchain configuration, including RPC URL and wallet credentials.
    pub blockchain: BlockchainConfig,
    pub injector: InjectorConfig,
}

/// Blockchain configuration, including RPC URL and wallet credentials.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BlockchainConfig {
    /// The address of the EAS contract on the L2 chain.
    pub eas_address: Address,
    /// The address of the L2AnchoringManager contract on the L2 chain.
    pub manager_address: Address,
    /// The address of the FeeOracle contract on the L2 chain.
    pub fee_oracle_address: Address,
    /// Rpc configuration.
    pub rpc: RpcConfig,
    /// Wallet configuration, including mnemonic and index for key derivation.
    pub wallet: WalletConfig,
}

/// Rpc Config
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RpcConfig {
    /// The RPC URL of the Ethereum L2 node to connect to.
    pub l2: String,
    /// Retry backoff configuration for handling rate-limited requests to the RPC provider.
    #[serde(default)]
    pub retry: RetryBackoffArgs,
    /// Throttle configuration for limiting the rate of requests to the RPC provider.
    #[serde(default)]
    pub throttle: ThrottleArgs,
}

/// Wallet configuration, including mnemonic and index for key derivation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct WalletConfig {
    /// The mnemonic phrase for the wallet, used to derive the signing key.
    pub mnemonic: String,
    /// The index for key derivation from the mnemonic, following BIP-44.
    pub index: u32,
}

/// Bot configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct InjectorConfig {
    pub drand_base_url: Url,
    pub calendar_url: Url,
}

impl AppConfig {
    /// Load configuration from defaults, a config file, and environment variables.
    pub fn new() -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(
                Environment::with_prefix("CALENDAR")
                    .separator("_")
                    .try_parsing(true),
            )
            .build()?;

        settings.try_deserialize()
    }
}
