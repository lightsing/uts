use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

/// Application configuration loaded from defaults, a config file, and environment variables.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    /// Server configuration, including node name and bind address.
    pub server: ServerConfig,
    /// Blockchain configuration, including RPC URL and wallet credentials.
    pub blockchain: BlockchainConfig,
    /// Database configuration for journal, key-value store, and SQL database.
    pub db: DbConfig,
    /// Stamper configuration, including timing and entry limits.
    pub stamper: StamperConfig,
}

/// Server configuration, including node name and bind address.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ServerConfig {
    /// A human-readable name for the calendar node, used in homepage.
    pub node_name: String,
    /// The address and port to bind the server to.
    pub bind_address: String,
}

/// Blockchain configuration, including RPC URL and wallet credentials.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BlockchainConfig {
    /// The RPC URL of the Ethereum node to connect to.
    pub rpc_url: String,
    /// Wallet configuration, including mnemonic and index for key derivation.
    pub wallet: WalletConfig,
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

/// Database configuration for journal, key-value store, and SQL database.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DbConfig {
    /// Configuration for the journal, including capacity and database path.
    pub journal: JournalConfig,
    /// Configuration for the key-value store, including storage path.
    pub kv: KvConfig,
    /// Configuration for the SQL database, including filename.
    pub sql: SqlConfig,
}

/// Configuration for the journal, including capacity and database path.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct JournalConfig {
    /// The maximum number of entries the journal can hold before old entries are overwritten.
    pub capacity: usize,
    /// The file system path where the journal database is stored.
    pub db_path: PathBuf,
}

/// Configuration for the key-value store, including storage path.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct KvConfig {
    /// The file system path where the key-value store (e.g. RocksDB) is stored.
    pub path: PathBuf,
}

/// Configuration for the SQL database, including filename.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SqlConfig {
    pub filename: PathBuf,
}

/// Configuration for the Stamper, including timing and entry limits.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StamperConfig {
    /// See [`uts_stamper::StamperConfig`].
    pub max_interval_seconds: u64,
    /// See [`uts_stamper::StamperConfig`].
    pub max_entries_per_timestamp: usize,
    /// See [`uts_stamper::StamperConfig`].
    pub min_leaves: usize,
}

impl AppConfig {
    /// Load configuration from defaults, a config file, and environment variables.
    pub fn new() -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .set_default("server.bind-address", "0.0.0.0:3000")?
            .set_default("db.journal.capacity", 1048576u32)?
            .set_default("db.journal.db-path", "./.db/journal")?
            .set_default("db.kv.path", "./.db/tries")?
            .set_default("db.sql.filename", "./.db/calendar.sqlite")?
            .set_default("stamper.max-interval-seconds", 10u64)?
            .set_default("stamper.max-entries-per-timestamp", 1024u32)?
            .set_default("stamper.min-leaves", 16u32)?
            .add_source(File::with_name("config").required(false))
            .add_source(
                Environment::with_prefix("CALENDAR")
                    .separator("_")
                    .try_parsing(true), // 尝试将字符串解析为对应的类型 (如 usize, u64)
            )
            .build()?;

        settings.try_deserialize()
    }
}
