use crate::{
    codec::v1::{Attestation, PendingAttestation, RawAttestation},
    error::DecodeError,
};

#[cfg(feature = "ethereum-uts-verifier")]
mod ethereum_uts;
#[cfg(feature = "ethereum-uts-verifier")]
pub use ethereum_uts::EthereumUTSVerifier;

#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    /// The raw attestation lacks a value, so it cannot be verified.
    #[error("raw attestation lacks a value")]
    NoValue,
    /// The attestation is still pending and cannot be verified yet.
    #[error("attestation is still pending and cannot be verified yet")]
    Pending,
    /// The attestation is not the expected type
    /// (e.g. a Bitcoin attestation was expected but an Ethereum attestation was found).
    #[error("attestation is not the expected type")]
    BadAttestationTag,
    /// An error occurred while decoding the attestation.
    #[error("error decoding attestation: {0}")]
    Decode(DecodeError),
    /// An error occurred while verifying the attestation.
    #[error("error verifying attestation: {0}")]
    Verify(#[source] Box<dyn core::error::Error + Send + Sync>),
}

pub trait AttestationVerifier<P>
where
    P: for<'a> Attestation<'a> + Send,
    Self: Send + Sync,
{
    fn verify_raw(
        &self,
        raw: &RawAttestation,
    ) -> impl Future<Output = Result<(), VerifyError>> + Send {
        async {
            if raw.tag == PendingAttestation::TAG {
                return Err(VerifyError::Pending);
            }

            let Some(value) = raw.value.get() else {
                return Err(VerifyError::NoValue);
            };

            match P::from_raw(raw) {
                Ok(attestation) => self.verify(&attestation, value).await,
                Err(DecodeError::BadAttestationTag) => Err(VerifyError::BadAttestationTag),
                Err(e) => Err(VerifyError::Decode(e)),
            }
        }
    }

    fn verify(
        &self,
        attestation: &P,
        value: &[u8],
    ) -> impl Future<Output = Result<(), VerifyError>> + Send;
}
