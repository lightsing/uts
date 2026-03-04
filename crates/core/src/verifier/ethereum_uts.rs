use super::{AttestationVerifier, VerifyError};
use crate::codec::v1::EthereumUTSAttestation;
use alloy_primitives::{Address, ChainId, TxHash};
use alloy_provider::{Provider, transport::TransportError};
use alloy_rpc_types_eth::{Filter, Log};
use alloy_sol_types::SolEvent;
use digest::OutputSizeUser;
use sha3::Keccak256;
use uts_contracts::uts::Attested;

#[derive(Debug, Clone)]
pub struct EthereumUTSVerifier<P: Provider> {
    provider: P,
    chain_id: ChainId,
}

#[derive(Debug, thiserror::Error)]
pub enum EthereumUTSVerifierError {
    #[error("invalid value length for Ethereum UTS attestation")]
    InvalidLength,
    #[error("chain ID mismatch")]
    ChainIdMismatch,
    #[error("root not found in attested logs")]
    NotFound,
    #[error("contract address mismatch, expected {expected}, found {found}")]
    ContractMismatch { expected: Address, found: Address },
    #[error("transaction hash mismatch, expected {expected}, found {found}")]
    TransactionMismatch { expected: TxHash, found: TxHash },
    #[error(transparent)]
    Rpc(#[from] TransportError),
}

impl<P: Provider> EthereumUTSVerifier<P> {
    pub async fn new(provider: P) -> Result<Self, EthereumUTSVerifierError> {
        let chain_id = provider.get_chain_id().await?;
        Ok(Self { provider, chain_id })
    }
}

impl<P: Provider> AttestationVerifier<EthereumUTSAttestation> for EthereumUTSVerifier<P> {
    type Output = Log<Attested>;

    async fn verify(
        &self,
        attestation: &EthereumUTSAttestation,
        value: &[u8],
    ) -> Result<Self::Output, VerifyError> {
        Ok(self.verify_attestation(attestation, value).await?)
    }
}

impl<P: Provider> EthereumUTSVerifier<P> {
    async fn verify_attestation(
        &self,
        attestation: &EthereumUTSAttestation,
        value: &[u8],
    ) -> Result<Log<Attested>, EthereumUTSVerifierError> {
        if value.len() != Keccak256::output_size() {
            return Err(EthereumUTSVerifierError::InvalidLength);
        }
        if attestation.chain.id() != self.chain_id {
            return Err(EthereumUTSVerifierError::ChainIdMismatch);
        }

        let filter = Filter::new()
            .from_block(attestation.height)
            .to_block(attestation.height)
            .event_signature(Attested::SIGNATURE_HASH);
        let logs = self.provider.get_logs(&filter).await?;

        let Some(log) = logs
            .into_iter()
            .filter_map(|log| {
                Attested::decode_log(&log.inner)
                    .map(|inner| Log {
                        inner,
                        block_hash: log.block_hash,
                        block_number: log.block_number,
                        block_timestamp: log.block_timestamp,
                        transaction_hash: log.transaction_hash,
                        transaction_index: log.transaction_index,
                        log_index: log.log_index,
                        removed: log.removed,
                    })
                    .ok()
            })
            .find(|log| log.inner.data.root == value)
        else {
            return Err(EthereumUTSVerifierError::NotFound);
        };

        // perform additional checks if available
        if let Some(contract) = attestation.metadata.contract() {
            if log.inner.address != contract {
                return Err(EthereumUTSVerifierError::ContractMismatch {
                    expected: contract,
                    found: log.inner.address,
                });
            }
            if let Some(expect_tx) = attestation.metadata.tx()
                && let Some(found_tx) = log.transaction_hash
                && expect_tx != found_tx
            {
                return Err(EthereumUTSVerifierError::TransactionMismatch {
                    expected: expect_tx,
                    found: found_tx,
                });
            }
        }
        Ok(log)
    }
}

impl EthereumUTSVerifierError {
    /// The error indicates this attestation is invalid and cannot be verified.
    #[inline]
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            EthereumUTSVerifierError::InvalidLength | EthereumUTSVerifierError::NotFound
        )
    }

    /// The error indicates this attestation is valid but not attested by the expected contract or transaction.
    #[inline]
    pub fn is_mismatch(&self) -> bool {
        matches!(
            self,
            EthereumUTSVerifierError::ContractMismatch { .. }
                | EthereumUTSVerifierError::TransactionMismatch { .. }
        )
    }

    /// The error indicates this attestation may be valid but cannot be verified at the moment.
    #[inline]
    pub fn should_retry(&self) -> bool {
        matches!(self, EthereumUTSVerifierError::Rpc(_))
    }

    /// The error indicates this attestation may be valid but the provider is not suitable for verifying it.
    #[inline]
    pub fn is_wrong_provider(&self) -> bool {
        matches!(self, EthereumUTSVerifierError::ChainIdMismatch)
    }
}
