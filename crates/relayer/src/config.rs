use alloy_primitives::{Address, BlockNumber, U256};
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;
use uts_contracts::provider_helper::{RetryBackoffArgs, ThrottleArgs};

/// Application configuration loaded from defaults, a config file, and environment variables.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    /// Blockchain configuration, including RPC URL and wallet credentials.
    pub blockchain: BlockchainConfig,
    /// Indexer configuration.
    pub indexer: IndexerConfig,
    /// Relayer configuration.
    pub relayer: RelayerConfig,
    /// Database configuration for journal, key-value store, and SQL database.
    pub db: DbConfig,
}

/// Blockchain configuration, including RPC URL and wallet credentials.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BlockchainConfig {
    /// The address of the L2AnchoringManager contract on the L2 chain.
    pub manager_address: Address,
    /// The address of the L1AnchoringGateway contract on the L1 chain.
    pub gateway_address: Address,
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
    pub l1: String,
    /// The RPC URL of the Ethereum L2 node to connect to.
    pub l2_ws: String,
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

/// Indexer configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct IndexerConfig {
    /// Configuration for the L2 indexer.
    pub l2: L2IndexerConfig,
}

/// Configuration for the L2 indexer.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct L2IndexerConfig {
    /// The block number to start indexing from. This should be the block number of the deployment
    /// transaction of the L2AnchoringManager contract, or a safe block number before that to ensure
    /// we don't miss any events.
    pub start_block: BlockNumber,
    /// The batch size for fetching events. This determines how many blocks to fetch in each query
    /// when calling `eth_getLogs`.
    pub batch_size: u64,
}

/// Configuration for the Relayer
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RelayerConfig {
    /// The maximum number of L2 anchoring events to include in a single batch when sending to L1.
    pub batch_max_size: i64,
    /// The maximum time in seconds to wait before sealing a batch and sending it to L1.
    pub batch_max_wait_seconds: i64,
    /// The gas limit to use for submitting a batch to L1.
    pub l1_batch_submission_gas_limit: u64,
    /// The fee in wei to pay for submitting a batch to L1.
    pub l1_batch_submission_fee: U256,
    /// The interval in seconds at which the relayer's main loop runs.
    pub tick_interval_seconds: u64,
}

/// Database configuration for journal, key-value store, and SQL database.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DbConfig {
    /// Configuration for the SQL database, including filename.
    pub sql: SqlConfig,
}

/// Configuration for the SQL database, including filename.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SqlConfig {
    /// The file system path where the SQL database (e.g. SQLite) is stored.
    pub filename: PathBuf,
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
