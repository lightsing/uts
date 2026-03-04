use crate::client::CLIENT;
use bytemuck::Pod;
use clap::{Args, ValueEnum};
use digest::{Digest, FixedOutputReset, Output};
use futures::TryFutureExt;
use std::{collections::HashMap, future::ready, io, path::PathBuf, sync::LazyLock, time::Duration};
use tokio::{fs, io::AsyncWriteExt};
use url::Url;
use uts_bmt::UnorderedMerkleTree;
use uts_core::{
    codec::{
        Decode, Encode, VersionedProof,
        v1::{DetachedTimestamp, DigestHeader, Timestamp, TimestampBuilder, opcode::DigestOpExt},
    },
    utils::{HashAsyncFsExt, Hexed},
};

static DEFAULT_CALENDARS: LazyLock<Vec<Url>> = LazyLock::new(|| {
    vec![
        // Url::parse("https://a.pool.opentimestamps.org/").unwrap(),
        // Url::parse("https://b.pool.opentimestamps.org/").unwrap(),
        // Url::parse("https://a.pool.eternitywall.com/").unwrap(),
        // Url::parse("https://ots.btc.catallaxy.com/").unwrap(),
        Url::parse("http://127.0.0.1:3000/").unwrap(),
    ]
});

#[derive(Debug, Args)]
pub struct Stamp {
    /// Files to timestamp. May be specified multiple times.
    #[arg(value_name = "FILE", num_args = 1..)]
    files: Vec<PathBuf>,
    /// Create timestamp with the aid of a remote calendar. May be specified multiple times.
    #[arg(short = 'c', long = "calendar", value_name = "URL", num_args = 0..)]
    calendars: Vec<Url>,
    /// Consider the timestamp complete if at least M calendars reply prior to the timeout
    #[arg(short = 'm', default_value = "1")]
    quorum: usize,
    /// Hasher to use when digesting files. Default is Keccak256.
    #[arg(short = 'H', long = "hasher", default_value = "keccak256")]
    hasher: Hasher,
    /// Timeout in seconds to wait for calendar responses. Default is 60 seconds.
    #[arg(long = "timeout", default_value = "5")]
    timeout: u64,
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
        Output<D>: Pod + Copy,
    {
        let digests =
            futures::future::join_all(self.files.iter().map(|f| hash_file::<D>(f.clone())))
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;

        for (header, path) in digests.iter().zip(self.files.iter()) {
            eprintln!("File: {} {}", header, path.display());
        }

        let mut builders: HashMap<PathBuf, TimestampBuilder> = HashMap::from_iter(
            self.files
                .iter()
                .map(|path| (path.clone(), Timestamp::builder())),
        );

        let nonced_digest = builders
            .iter_mut()
            .zip(digests.iter())
            .map(|((_, builder), digest)| {
                let mut hasher = D::new();
                Digest::update(&mut hasher, digest.digest());
                let nonce: [u8; 32] = rand::random();
                Digest::update(&mut hasher, &nonce);
                builder.append(nonce.to_vec()).digest::<D>();
                hasher.finalize()
            })
            .collect::<Vec<_>>();

        let internal_tire = UnorderedMerkleTree::<D>::new(&nonced_digest);
        let root = internal_tire.root();
        eprintln!("Internal Merkle root: {}", Hexed(root));

        for ((_, builder), leaf) in builders.iter_mut().zip(nonced_digest) {
            let proof = internal_tire.get_proof_iter(&leaf).expect("infallible");
            builder.merkle_proof(proof);
        }

        let calendars = if self.calendars.is_empty() {
            &*DEFAULT_CALENDARS
        } else {
            &*self.calendars
        };

        if self.quorum > calendars.len() {
            eyre::bail!(
                "Quorum of {} cannot be achieved with only {} calendars",
                self.quorum,
                self.calendars.len()
            );
        }

        let stamps = futures::future::join_all(
            calendars
                .into_iter()
                .map(|calendar| request_calendar(calendar.clone(), self.timeout, root)),
        )
        .await
        .into_iter()
        .filter_map(|res| res.ok())
        .collect::<Vec<_>>();
        if stamps.len() < self.quorum {
            eyre::bail!(
                "Only received {} valid responses from calendars, which does not meet the quorum of {}",
                stamps.len(),
                self.quorum
            );
        }
        let merged = if stamps.len() == 1 {
            stamps.into_iter().next().unwrap()
        } else {
            Timestamp::merge(stamps)
        };

        let writes =
            futures::future::join_all(builders.into_iter().zip(digests).map(
                |((path, builder), header)| write_stamp(path, builder, merged.clone(), header),
            ))
            .await;
        for (res, path) in writes.into_iter().zip(self.files.iter()) {
            match res {
                Ok(_) => eprintln!("Successfully wrote timestamp for {}", path.display()),
                Err(e) => eprintln!("Failed to write timestamp for {}: {}", path.display(), e),
            }
        }

        Ok(())
    }
}

async fn hash_file<D: DigestOpExt + Send>(path: PathBuf) -> io::Result<DigestHeader> {
    let mut hasher = D::new();
    let file = fs::File::open(path).await?;
    HashAsyncFsExt::update(&mut hasher, file).await?;
    Ok(DigestHeader::new::<D>(hasher.finalize()))
}

async fn request_calendar(calendar: Url, timeout: u64, root: &[u8]) -> eyre::Result<Timestamp> {
    eprintln!("Submitting to remote calendar: {calendar}");
    let url = calendar.join(&"digest")?;
    let response = CLIENT
        .post(url)
        .header("Accept", "application/vnd.opentimestamps.v1")
        .body(root.to_vec())
        .timeout(Duration::from_secs(timeout))
        .send()
        .and_then(|r| ready(r.error_for_status()))
        .and_then(|r| r.bytes())
        .await
        .inspect_err(|e| {
            if e.is_status() {
                eprintln!("Calendar {} responded with error: {}", calendar, e);
            } else if e.is_timeout() {
                eprintln!("Calendar {} timed out after {} seconds", calendar, timeout);
            } else {
                eprintln!("Failed to submit to calendar {}: {}", calendar, e);
            }
        })?;

    let ts = Timestamp::decode(&mut &*response).inspect_err(|e| {
        eprintln!(
            "Failed to decode response from calendar {}: {}",
            calendar, e
        );
    })?;
    Ok(ts)
}

async fn write_stamp(
    mut path: PathBuf,
    builder: TimestampBuilder,
    merged: Timestamp,
    header: DigestHeader,
) -> eyre::Result<()> {
    let timestamp = builder.concat(merged.clone());
    let timestamp = DetachedTimestamp::from_parts(header, timestamp);
    let timestamp = VersionedProof::<DetachedTimestamp>::new(timestamp);
    let mut buf = Vec::new();
    timestamp.encode(&mut buf)?;
    path.add_extension("ots");
    let mut file = fs::File::create_new(path).await?;
    file.write_all(&buf).await?;
    Ok(())
}
