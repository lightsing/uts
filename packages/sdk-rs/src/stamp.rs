use crate::{Result, Sdk, error::Error};
use digest::{Digest, FixedOutputReset, Output};
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, instrument};
use url::Url;
use uts_bmt::MerkleTree;
use uts_core::{
    alloc,
    alloc::{Allocator, Global},
    codec::{
        DecodeIn,
        v1::{DetachedTimestamp, DigestHeader, Timestamp, TimestampBuilder, opcode::DigestOpExt},
    },
    utils::{HashAsyncFsExt, Hexed},
};

impl Sdk {
    /// Creates a timestamp for the given files.
    pub async fn stamp_files<D>(&self, files: &[PathBuf]) -> Result<Vec<DetachedTimestamp>>
    where
        D: Digest + FixedOutputReset + DigestOpExt + Send,
        Output<D>: Copy,
    {
        Ok(Vec::from_iter(
            self.stamp_files_in::<_, D>(files, Global).await?,
        ))
    }

    /// Creates a timestamp for the given digests.
    pub async fn stamp_digest<D>(&self, digests: &[Output<D>]) -> Result<Vec<DetachedTimestamp>>
    where
        D: Digest + FixedOutputReset + DigestOpExt + Send,
        Output<D>: Copy,
    {
        Ok(Vec::from_iter(
            self.stamp_digests_in::<_, D>(digests, Global).await?,
        ))
    }

    /// Creates a timestamp for the given files in the provided allocator.
    ///
    /// # Note
    ///
    /// This uses the `allocator_api2` crate for allocator api.
    pub async fn stamp_files_in<A, D>(
        &self,
        files: &[PathBuf],
        allocator: A,
    ) -> Result<alloc::vec::Vec<DetachedTimestamp<A>, A>>
    where
        A: Allocator + Clone,
        D: Digest + FixedOutputReset + DigestOpExt + Send,
        Output<D>: Copy,
    {
        let digests = futures::future::join_all(files.iter().map(|f| hash_file::<D>(f.clone())))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        self.stamp_digests_in::<_, D>(&digests, allocator).await
    }

    /// Creates a timestamp for the given digests in the provided allocator.
    ///
    /// # Note
    ///
    /// This uses the `allocator_api2` crate for allocator api.
    pub async fn stamp_digests_in<A, D>(
        &self,
        digests: &[Output<D>],
        allocator: A,
    ) -> Result<alloc::vec::Vec<DetachedTimestamp<A>, A>>
    where
        A: Allocator + Clone,
        D: Digest + FixedOutputReset + DigestOpExt + Send,
        Output<D>: Copy,
    {
        let mut builders: alloc::vec::Vec<TimestampBuilder<A>, A> =
            alloc::vec![in allocator.clone(); Timestamp::builder_in(allocator.clone()) ];

        let mut nonced_digest = alloc::vec::Vec::with_capacity_in(digests.len(), allocator.clone());

        for (builder, digest) in builders.iter_mut().zip(digests.iter()) {
            if self.inner.nonce_size == 0 {
                nonced_digest.push(*digest);
                continue;
            }

            let mut hasher = D::new();
            Digest::update(&mut hasher, digest);

            let mut nonce =
                alloc::vec::Vec::with_capacity_in(self.inner.nonce_size, allocator.clone());
            nonce.resize(self.inner.nonce_size, 0);
            rand::fill(&mut nonce[..]);

            Digest::update(&mut hasher, &nonce);
            builder.append(nonce).digest::<D>();

            nonced_digest.push(hasher.finalize())
        }

        let root = if digests.len() > 1 {
            let internal_tire = MerkleTree::<D>::new(&nonced_digest);
            let root = internal_tire.root();
            debug!(internal_tire_root = ?Hexed(root));

            for (builder, leaf) in builders.iter_mut().zip(nonced_digest) {
                let proof = internal_tire.get_proof_iter(&leaf).expect("infallible");
                builder.merkle_proof(proof);
            }
            *root
        } else {
            nonced_digest[0]
        };

        let stamps_futures = futures::future::join_all(
            self.inner
                .calendars
                .iter()
                .map(|calendar| self.request_calendar(calendar.clone(), &root, allocator.clone())),
        )
        .await
        .into_iter()
        .filter_map(|res| res.ok());
        let mut results =
            alloc::vec::Vec::with_capacity_in(self.inner.calendars.len(), allocator.clone());
        for stamp in stamps_futures {
            results.push(stamp);
        }

        if results.len() < self.inner.quorum {
            return Err(Error::QuorumNotReached {
                required: self.inner.quorum,
                received: results.len(),
            });
        }

        let merged = if results.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            Timestamp::<A>::merge_in(results, allocator.clone())
        };

        let mut stamps = alloc::vec::Vec::with_capacity_in(builders.len(), allocator.clone());
        for (builder, digest) in builders.into_iter().zip(digests.iter()) {
            let timestamp = builder.concat(merged.clone());
            let header = DigestHeader::new::<D>(*digest);
            let timestamp = DetachedTimestamp::from_parts(header, timestamp);
            stamps.push(timestamp);
        }

        Ok(stamps)
    }

    #[instrument(skip(self, allocator), level = "debug", err)]
    async fn request_calendar<A: Allocator + Clone>(
        &self,
        calendar: Url,
        root: &[u8],
        allocator: A,
    ) -> Result<Timestamp<A>> {
        let url = calendar.join("digest")?;

        let root = root.to_vec();
        let (_, body) = self
            .http_request_with_retry(
                url,
                10 * 1024, // 10 KB
                move |req| {
                    req.header("Accept", "application/vnd.opentimestamps.v1")
                        .body(root.clone())
                },
            )
            .await?;

        Ok(Timestamp::<A>::decode_in(&mut &*body, allocator)?)
    }
}

async fn hash_file<D: DigestOpExt + Send>(path: PathBuf) -> Result<Output<D>> {
    let mut hasher = D::new();
    let file = fs::File::open(path).await?;
    HashAsyncFsExt::update(&mut hasher, file).await?;
    Ok(hasher.finalize())
}
