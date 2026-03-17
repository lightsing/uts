use crate::{
    codec::v1::BitcoinAttestation,
    verifier::{AttestationVerifier, VerifyError},
};
use alloy_primitives::{hex, hex::FromHexError};
use backon::{ExponentialBuilder, Retryable};
use http_body_util::LengthLimitError;
use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use tracing::instrument;
use url::Url;

const RESPONSE_SIZE_LIMIT: usize = 10 * 1024; // 10 KiB

#[derive(Debug, Clone)]
pub struct BitcoinVerifier {
    client: Client,
    provider: Url,
    retry: ExponentialBuilder,
}

#[derive(Debug, thiserror::Error)]
pub enum BitcoinVerifierError {
    #[error("error making JSON-RPC request: {0}")]
    Request(#[from] reqwest::Error),
    #[error("error processing JSON-RPC response: {0}")]
    Response(Box<dyn std::error::Error + Send + Sync>),
    #[error("error parsing JSON-RPC response: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Rpc(JsonRpcError),
    #[error("invalid hex")]
    Hex(#[from] FromHexError),
    #[error("invalid attestation")]
    Invalid,
}

#[derive(Debug, Deserialize, thiserror::Error)]
#[error("rpc error {code}: {message}")]
pub struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BitcoinHeader {
    pub hash: String,
    pub merkleroot: String,
    pub height: u32,
    pub time: u64,
}

#[derive(Deserialize)]
struct JsonRpcResponse<T> {
    result: T,
    error: Option<JsonRpcError>,
}

impl BitcoinVerifier {
    pub fn new(provider: Url) -> Self {
        Self {
            client: Client::new(),
            provider,
            retry: ExponentialBuilder::default(),
        }
    }

    pub fn from_parts(client: Client, provider: Url, retry: ExponentialBuilder) -> Self {
        Self {
            client,
            provider,
            retry,
        }
    }

    pub async fn get_blockhash(&self, height: u32) -> Result<String, BitcoinVerifierError> {
        self.req("getblockhash", [height]).await
    }

    pub async fn get_block_header(
        &self,
        hash: &str,
    ) -> Result<BitcoinHeader, BitcoinVerifierError> {
        self.req("getblockheader", (hash, true)).await
    }

    pub async fn verify(
        &self,
        attestation: &BitcoinAttestation,
        value: &[u8],
    ) -> Result<BitcoinHeader, BitcoinVerifierError> {
        let height = attestation.height;

        let blockhash = self.get_blockhash(height).await?;

        let header = self.get_block_header(&blockhash).await?;

        // Bitcoin reverses the blockhash in RPC responses, so we need to reverse it back to get the correct hash.
        let mut hash = hex::decode(&header.merkleroot)?;
        hash.reverse();
        if hash != value {
            return Err(BitcoinVerifierError::Invalid);
        }

        Ok(header)
    }

    #[instrument(skip(self, params), level = "trace", err(level = "warn"))]
    async fn req<P: Serialize, T: DeserializeOwned>(
        &self,
        method: &str,
        params: P,
    ) -> Result<T, BitcoinVerifierError> {
        let body = json!({
            "jsonrpc": "1.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        let req = self.client.post(self.provider.clone()).json(&body);

        {
            move || {
                let req = req.try_clone().expect("infallible");

                async move {
                    let res = req.send().await?.error_for_status()?;

                    let res: http::Response<reqwest::Body> = res.into();
                    let body = http_body_util::Limited::new(res.into_body(), RESPONSE_SIZE_LIMIT);
                    let bytes = http_body_util::BodyExt::collect(body)
                        .await
                        .map_err(BitcoinVerifierError::Response)?
                        .to_bytes();

                    let response: JsonRpcResponse<T> = serde_json::from_slice(&bytes)?;
                    if let Some(error) = response.error {
                        Err(BitcoinVerifierError::Rpc(error))
                    } else {
                        Ok(response.result)
                    }
                }
            }
        }
        .retry(self.retry)
        .when(|e| {
            use BitcoinVerifierError::*;
            match e {
                Request(_) => true,
                Response(e) => e.downcast_ref::<LengthLimitError>().is_none(),
                _ => false,
            }
        })
        .await
    }
}

impl AttestationVerifier<BitcoinAttestation> for BitcoinVerifier {
    type Output = BitcoinHeader;

    async fn verify(
        &self,
        attestation: &BitcoinAttestation,
        value: &[u8],
    ) -> Result<Self::Output, VerifyError> {
        Ok(self.verify(attestation, value).await?)
    }
}

impl BitcoinVerifierError {
    /// The error indicates this attestation is invalid and cannot be verified.
    #[inline]
    pub fn is_fatal(&self) -> bool {
        matches!(self, BitcoinVerifierError::Invalid)
    }

    /// The error indicates this attestation may be valid but cannot be verified at the moment.
    #[inline]
    pub fn should_retry(&self) -> bool {
        !matches!(self, BitcoinVerifierError::Invalid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_get_blockhash() {
        let verifier =
            BitcoinVerifier::new(Url::parse("https://bitcoin-rpc.publicnode.com").unwrap());
        let hash = verifier.get_blockhash(0).await.unwrap();
        assert_eq!(
            hash,
            "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
        );
        let header = verifier.get_block_header(&hash).await.unwrap();
        assert_eq!(header.hash, hash);
        assert_eq!(header.height, 0);
        assert_eq!(header.time, 1231006505);
    }
}
