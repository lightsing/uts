use clap::{Args, ValueEnum};
use digest::{Digest, FixedOutputReset, Output};
use futures::future::join_all;
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};
use tracing::{error, info};
use url::Url;
use uts_core::codec::{
    Encode, VersionedProof,
    v1::{DetachedTimestamp, opcode::DigestOpExt},
};
use uts_sdk::Sdk;

#[derive(Debug, Args)]
pub struct Stamp {
    /// Files to timestamp. May be specified multiple times.
    #[arg(value_name = "FILE", num_args = 1..)]
    files: Vec<PathBuf>,
    /// Create timestamp with the aid of a remote calendar. May be specified multiple times.
    #[arg(short = 'c', long = "calendar", value_name = "URL", num_args = 0..)]
    calendars: Vec<Url>,
    /// Consider the timestamp complete if at least M calendars reply prior to the timeout
    #[arg(short = 'm')]
    quorum: Option<usize>,
    /// Hasher to use when digesting files. Default is Keccak256.
    #[arg(short = 'H', long = "hasher", default_value = "keccak256")]
    hasher: Hasher,
    /// Timeout in seconds to wait for calendar responses
    #[arg(long = "timeout")]
    timeout: Option<u64>,
}

#[derive(Default, Debug, Copy, Clone, ValueEnum)]
pub enum Hasher {
    Sha1,
    Ripemd160,
    Sha256,
    #[default]
    Keccak256,
}

impl Stamp {
    pub async fn run(self) -> eyre::Result<()> {
        match self.hasher {
            Hasher::Sha1 => self.run_inner::<sha1::Sha1>().await,
            Hasher::Ripemd160 => self.run_inner::<ripemd::Ripemd160>().await,
            Hasher::Sha256 => self.run_inner::<sha2::Sha256>().await,
            Hasher::Keccak256 => self.run_inner::<sha3::Keccak256>().await,
        }
    }

    async fn run_inner<D>(self) -> eyre::Result<()>
    where
        D: Digest + FixedOutputReset + DigestOpExt + Send,
        Output<D>: Copy,
    {
        let mut builder = if self.calendars.is_empty() {
            Sdk::builder()
        } else {
            Sdk::try_builder_from_calendars(self.calendars).expect("none empty")
        };
        if let Some(quorum) = self.quorum {
            builder = builder.with_quorum(quorum)
        }
        if let Some(timeout) = self.timeout {
            builder = builder.with_timeout_seconds(timeout)
        }

        let sdk = builder.build()?;

        let stamps = sdk.stamp_files::<D>(&self.files).await?;

        let tasks = join_all(
            self.files
                .clone()
                .into_iter()
                .zip(stamps)
                .map(|(path, timestamp)| write_stamp(path, timestamp)),
        )
        .await;

        for (path, result) in self.files.iter().zip(tasks) {
            match result {
                Ok(()) => info!("[{}] successfully stamped", path.display()),
                Err(e) => error!("[{}] failed to stamp: {e}", path.display()),
            }
        }

        Ok(())
    }
}

async fn write_stamp(mut path: PathBuf, timestamp: DetachedTimestamp) -> eyre::Result<()> {
    let timestamp = VersionedProof::<DetachedTimestamp>::new(timestamp);
    let mut buf = Vec::new();
    timestamp.encode(&mut buf)?;
    path.add_extension("ots");
    let mut file = fs::File::create_new(path).await?;
    file.write_all(&buf).await?;
    Ok(())
}
