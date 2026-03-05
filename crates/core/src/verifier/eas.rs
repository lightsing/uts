use super::{AttestationVerifier, VerifyError};
use crate::codec::v1::{EASAttestation, EASTimestamped};
use alloy_primitives::{Address, B256};
use alloy_provider::Provider;
use alloy_sol_types::SolValue;
use uts_contracts::eas::{self, EAS};

#[derive(Debug, Clone)]
pub struct EASVerifier<P: Provider> {
    eas: EAS<P>,
}

#[derive(Debug, thiserror::Error)]
pub enum EASVerifierError {
    #[error("invalid value length for EAS attestation")]
    InvalidLength,
    #[error("invalid attestation data")]
    InvalidData(#[from] alloy_sol_types::Error),
    #[error("unexpected schema used for attestation")]
    InvalidSchema,
    #[error("attestation cannot be revocable")]
    RevocableAttestation,
    #[error("attested hash is not equal to the expected hash")]
    Mismatched { expected: B256, actual: B256 },
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    Rpc(#[from] alloy_contract::Error),
}

impl<P: Provider> EASVerifier<P> {
    pub const fn new(address: Address, provider: P) -> Self {
        let eas = EAS::new(address, provider);
        Self { eas }
    }
}

impl<P: Provider> AttestationVerifier<EASAttestation> for EASVerifier<P> {
    type Output = eas::Attestation;

    async fn verify(
        &self,
        attestation: &EASAttestation,
        value: &[u8],
    ) -> Result<Self::Output, VerifyError> {
        self.verify_attestation(attestation, value)
            .await
            .map_err(VerifyError::EAS)
    }
}

impl<P: Provider> AttestationVerifier<EASTimestamped> for EASVerifier<P> {
    type Output = u64;

    async fn verify(
        &self,
        attestation: &EASTimestamped,
        value: &[u8],
    ) -> Result<Self::Output, VerifyError> {
        self.verify_timestamped(attestation, value)
            .await
            .map_err(VerifyError::EAS)
    }
}

impl<P: Provider> EASVerifier<P> {
    async fn verify_attestation(
        &self,
        attestation: &EASAttestation,
        value: &[u8],
    ) -> Result<eas::Attestation, EASVerifierError> {
        let hash = B256::try_from(value).map_err(|_| EASVerifierError::InvalidLength)?;

        let attestation = self.eas.getAttestation(attestation.uid).call().await?;

        if attestation.schema != eas::SCHEMA_ID {
            return Err(EASVerifierError::InvalidSchema);
        }
        if attestation.revocable {
            return Err(EASVerifierError::RevocableAttestation);
        }

        let attested_hash = B256::abi_decode(&attestation.data)?;

        if attested_hash != hash {
            return Err(EASVerifierError::Mismatched {
                expected: hash,
                actual: attested_hash,
            });
        }

        Ok(attestation)
    }

    async fn verify_timestamped(
        &self,
        _attestation: &EASTimestamped,
        value: &[u8],
    ) -> Result<u64, EASVerifierError> {
        let hash = B256::try_from(value).map_err(|_| EASVerifierError::InvalidLength)?;

        let timestamp = self.eas.getTimestamp(hash).call().await?;

        if timestamp == 0 {
            return Err(EASVerifierError::NotFound);
        }

        Ok(timestamp)
    }
}

impl EASVerifierError {
    /// The error indicates this attestation is invalid and cannot be verified.
    #[inline]
    pub fn is_fatal(&self) -> bool {
        !matches!(self, EASVerifierError::Rpc(_))
    }

    /// The error indicates this attestation may be valid but cannot be verified at the moment.
    #[inline]
    pub fn should_retry(&self) -> bool {
        matches!(self, EASVerifierError::Rpc(_))
    }
}
