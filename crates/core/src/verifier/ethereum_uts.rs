use super::{AttestationVerifier, VerifyError};
use crate::codec::v1::EthereumUTSAttestation;
use alloy_primitives::ChainId;
use alloy_provider::{Provider, transport::TransportError};
use alloy_rpc_types_eth::Filter;
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
    #[error("different contract address in attested log than expected")]
    ContractAddressMismatch,
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
    async fn verify(
        &self,
        attestation: &EthereumUTSAttestation,
        value: &[u8],
    ) -> Result<(), VerifyError> {
        self.verify_attestation(attestation, value)
            .await
            .map_err(|e| VerifyError::Verify(Box::new(e)))
    }
}

impl<P: Provider> EthereumUTSVerifier<P> {
    async fn verify_attestation(
        &self,
        attestation: &EthereumUTSAttestation,
        value: &[u8],
    ) -> Result<(), EthereumUTSVerifierError> {
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
            .iter()
            .filter_map(|log| Attested::decode_log(&log.inner).ok())
            .find(|log| log.data.root == value)
        else {
            return Err(EthereumUTSVerifierError::NotFound);
        };

        // perform additional checks if available
        if let Some(contract_address) = attestation.metadata.contract() {
            if log.address != contract_address {
                return Err(EthereumUTSVerifierError::ContractAddressMismatch);
            }
        }
        Ok(())
    }
}
