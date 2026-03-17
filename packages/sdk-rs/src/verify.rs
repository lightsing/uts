use crate::{Error, Result, Sdk};
#[cfg(any(feature = "eas-verifier", feature = "bitcoin-verifier"))]
use backon::RetryableWithContext;
use digest::Digest;
use jiff::Timestamp;
use std::path::Path;
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};
use uts_core::{
    alloc,
    alloc::Allocator,
    codec::v1::{
        Attestation, DetachedTimestamp, PendingAttestation, RawAttestation,
        opcode::{KECCAK256, RIPEMD160, SHA1, SHA256},
    },
};
#[cfg(feature = "eas-verifier")]
use {
    alloy_provider::DynProvider,
    uts_contracts::eas::EAS_ADDRESSES,
    uts_core::codec::v1::{EASAttestation, EASTimestamped},
    uts_core::verifier::EASVerifier,
};
#[cfg(feature = "bitcoin-verifier")]
use {uts_core::codec::v1::BitcoinAttestation, uts_core::verifier::BitcoinVerifier};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AttestationStatusKind {
    /// The attestation is valid.
    Valid(Timestamp),
    /// The attestation is invalid.
    Invalid,
    /// The attestation is pending and has not yet been verified.
    Pending,
    /// The attestation is unknown, either because it is of an unsupported type or because an error occurred during verification.
    Unknown,
}

#[derive(Debug, Clone)]
pub struct AttestationStatus<A: Allocator = alloc::Global> {
    pub attestation: RawAttestation<A>,
    pub status: AttestationStatusKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VerifyStatus {
    /// The timestamp is valid and all attestations are valid.
    Valid(Timestamp),
    /// The timestamp is partially valid, at least one attestation is not valid.
    PartiallyValid(Timestamp),
    /// All attestations are pending.
    Pending,
    /// All attestations are unknown.
    Unknown,
}

impl Sdk {
    /// Verifies the given file against the given detached timestamp, returning a list of attestation statuses.
    pub async fn verify(
        &self,
        file: impl AsRef<Path>,
        timestamp: &DetachedTimestamp,
    ) -> Result<Vec<AttestationStatus>> {
        Ok(Vec::from_iter(
            self.verify_in(file, timestamp, alloc::Global).await?,
        ))
    }

    /// Verifies the given file against the given detached timestamp, returning a list of attestation statuses.
    ///
    /// This is the same as `verify`, but allows specifying a custom allocator for the attestation statuses.
    ///
    /// # Note
    ///
    /// This uses the `allocator_api2` crate for allocator api.
    pub async fn verify_in<A: Allocator + Clone>(
        &self,
        file: impl AsRef<Path>,
        timestamp: &DetachedTimestamp<A>,
        allocator: A,
    ) -> Result<alloc::vec::Vec<AttestationStatus<A>, A>> {
        let digest_header = timestamp.header();
        match digest_header.kind().tag() {
            SHA1 => {
                self.verify_digest::<sha1::Sha1>(file, digest_header.digest())
                    .await
            }
            RIPEMD160 => {
                self.verify_digest::<ripemd::Ripemd160>(file, digest_header.digest())
                    .await
            }
            SHA256 => {
                self.verify_digest::<sha2::Sha256>(file, digest_header.digest())
                    .await
            }
            KECCAK256 => {
                self.verify_digest::<sha3::Keccak256>(file, digest_header.digest())
                    .await
            }
            _ => return Err(Error::Unsupported("unknown digest algorithm")),
        }?;

        timestamp.try_finalize()?;
        let mut result =
            alloc::vec::Vec::with_capacity_in(timestamp.attestations().count(), allocator);
        for attestation in timestamp.attestations() {
            let attestation = attestation.to_owned();

            if attestation.tag == PendingAttestation::TAG {
                result.push(AttestationStatus {
                    attestation,
                    status: AttestationStatusKind::Pending,
                });
                continue;
            }

            let status = self
                .verify_attestation_inner(&attestation)
                .await
                .unwrap_or(AttestationStatusKind::Unknown);

            result.push(AttestationStatus {
                attestation,
                status,
            });
        }

        Ok(result)
    }

    /// Verifies the digest of the given file against the expected digest.
    pub async fn verify_digest<D: Digest>(
        &self,
        file: impl AsRef<Path>,
        expected: &[u8],
    ) -> Result<()> {
        let mut file = BufReader::new(File::open(file.as_ref()).await?);
        let mut hasher = D::new();
        let mut buffer = [0u8; 64 * 1024]; // 64KB buffer
        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        let actual = hasher.finalize();

        if *actual != *expected {
            return Err(Error::DigestMismatch {
                expected: expected.to_vec().into_boxed_slice(),
                actual: actual.to_vec().into_boxed_slice(),
            });
        }
        Ok(())
    }

    /// Aggregate the individual attestation statuses into an overall verification status for the timestamp.
    ///
    /// The earliest valid attestation timestamp is used as the timestamp for the overall status, if there is at least one valid attestation.
    ///
    /// The logic is as follows:
    /// - If there is at least one VALID attestation:
    ///  - If there are also INVALID or UNKNOWN attestations, the overall status is PARTIAL_VALID
    ///  - Otherwise, the overall status is VALID
    /// - If there are no VALID attestations, but at least one PENDING attestation, the overall status is PENDING
    /// - If there are no VALID or PENDING attestations, the overall status is INVALID
    pub fn aggregate_verify_results(&self, results: &[AttestationStatus]) -> VerifyStatus {
        let mut valid_ts = None;
        let mut has_invalid = false;
        let mut has_unknown = false;
        let mut has_pending = false;

        for status in results {
            match status.status {
                AttestationStatusKind::Valid(ts) => {
                    if valid_ts.is_none() || ts < valid_ts.unwrap() {
                        valid_ts = Some(ts);
                    }
                }
                AttestationStatusKind::Invalid => has_invalid = true,
                AttestationStatusKind::Unknown => has_unknown = true,
                AttestationStatusKind::Pending => has_pending = true,
            }
        }

        if let Some(valid_ts) = valid_ts {
            if has_invalid || has_unknown {
                VerifyStatus::PartiallyValid(valid_ts)
            } else {
                VerifyStatus::Valid(valid_ts)
            }
        } else if has_pending {
            VerifyStatus::Pending
        } else {
            VerifyStatus::Unknown
        }
    }

    async fn verify_attestation_inner<A: Allocator + Clone>(
        &self,
        attestation: &RawAttestation<A>,
    ) -> Result<AttestationStatusKind, Error> {
        let _expected = attestation
            .value()
            .expect("Attestation value should be finalized");

        #[cfg(feature = "eas-verifier")]
        if attestation.tag == EASAttestation::TAG {
            let attestation = EASAttestation::from_raw(attestation)?;
            return self.verify_eas_attestation(_expected, attestation).await;
        } else if attestation.tag == EASTimestamped::TAG {
            let attestation = EASTimestamped::from_raw(attestation)?;
            return self.verify_eas_timestamped(_expected, attestation).await;
        }

        #[cfg(feature = "bitcoin-verifier")]
        if attestation.tag == BitcoinAttestation::TAG {
            let attestation = BitcoinAttestation::from_raw(attestation)?;
            return self.verify_bitcoin(_expected, attestation).await;
        }

        Ok(AttestationStatusKind::Unknown)
    }

    #[cfg(feature = "eas-verifier")]
    async fn verify_eas_attestation(
        &self,
        expected: &[u8],
        attestation: EASAttestation,
    ) -> Result<AttestationStatusKind> {
        let chain = attestation.chain;
        let provider = self
            .inner
            .eth_providers
            .get(&chain.id())
            .ok_or_else(|| Error::UnsupportedChain(chain.id()))?;
        let eas_address = EAS_ADDRESSES
            .get(&chain.id())
            .ok_or_else(|| Error::UnsupportedChain(chain.id()))?;

        let (_, result) = {
            |verifier: EASVerifier<DynProvider>| async {
                let res = verifier.verify_attestation(&attestation, expected).await;
                (verifier, res)
            }
        }
        .retry(self.inner.retry)
        .when(|e| e.should_retry())
        .context(EASVerifier::new(*eas_address, provider.clone()))
        .await;

        match result {
            Ok(result) => {
                let ts = Timestamp::from_second(result.time.try_into().expect("i64 overflow"))?;
                Ok(AttestationStatusKind::Valid(ts))
            }
            Err(e) if e.is_fatal() => Ok(AttestationStatusKind::Invalid),
            Err(_) => Ok(AttestationStatusKind::Unknown),
        }
    }

    #[cfg(feature = "eas-verifier")]
    async fn verify_eas_timestamped(
        &self,
        expected: &[u8],
        attestation: EASTimestamped,
    ) -> Result<AttestationStatusKind> {
        let chain = attestation.chain;
        let provider = self
            .inner
            .eth_providers
            .get(&chain.id())
            .ok_or_else(|| Error::UnsupportedChain(chain.id()))?;
        let eas_address = EAS_ADDRESSES
            .get(&chain.id())
            .ok_or_else(|| Error::UnsupportedChain(chain.id()))?;

        let (_, result) = {
            |verifier: EASVerifier<DynProvider>| async {
                let res = verifier.verify_timestamped(&attestation, expected).await;
                (verifier, res)
            }
        }
        .retry(self.inner.retry)
        .when(|e| e.should_retry())
        .context(EASVerifier::new(*eas_address, provider.clone()))
        .await;

        match result {
            Ok(time) => {
                let ts = Timestamp::from_second(time.try_into().expect("i64 overflow"))?;
                Ok(AttestationStatusKind::Valid(ts))
            }
            Err(e) if e.is_fatal() => Ok(AttestationStatusKind::Invalid),
            Err(_) => Ok(AttestationStatusKind::Unknown),
        }
    }

    #[cfg(feature = "bitcoin-verifier")]
    async fn verify_bitcoin(
        &self,
        expected: &[u8],
        attestation: BitcoinAttestation,
    ) -> Result<AttestationStatusKind> {
        let (_, result) = {
            |verifier: BitcoinVerifier| async {
                let res = verifier.verify(&attestation, expected).await;
                (verifier, res)
            }
        }
        .retry(self.inner.retry)
        .when(|e| e.should_retry())
        .context(BitcoinVerifier::from_parts(
            self.inner.http_client.clone(),
            self.inner.bitcoin_rpc.clone(),
            self.inner.retry,
        ))
        .await;

        match result {
            Ok(header) => {
                let ts = Timestamp::from_second(header.time.try_into().expect("i64 overflow"))?;
                Ok(AttestationStatusKind::Valid(ts))
            }
            Err(e) if e.is_fatal() => Ok(AttestationStatusKind::Invalid),
            Err(_) => Ok(AttestationStatusKind::Unknown),
        }
    }
}
