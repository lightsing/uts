//! Relayer for L1 anchoring.
#[macro_use]
extern crate tracing;

uts_sql::migrator!("./migrations");

/// Indexer.
pub mod indexer;

/// Relayer.
pub mod relayer;

pub(crate) mod sql;

/// Configuration.
pub mod config;
