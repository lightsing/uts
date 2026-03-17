use alloy_primitives::{ChainId, private::serde::de::StdError};

pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;

/// Error type for the SDK, encompassing various error scenarios that can occur during operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error during an I/O operation, such as reading or writing files.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Error during an HTTP request, such as a network failure or non-success status code.
    #[error("HTTP error: {0}")]
    Http(BoxError),
    /// Error parsing a URL, such as a calendar endpoint.
    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),
    /// Error during time-related operations, such as calculating timestamps or durations.
    #[error("Time error: {0}")]
    Jiff(#[from] jiff::Error),

    /// Error happened during decoding of a proof.
    #[error("uts decoding error: {0}")]
    Decode(#[from] uts_core::error::DecodeError),
    /// Error happened during encoding of a proof.
    #[error("uts encoding error: {0}")]
    Encode(#[from] uts_core::error::EncodeError),
    /// Error indicating that finalization of a timestamp failed due to conflicting inputs.
    #[error(transparent)]
    Finalization(#[from] uts_core::codec::v1::FinalizationError),

    /// Error indicating that a quorum of responses was not reached from the calendars.
    #[error("Quorum of {required} not reached, only {received} responses received")]
    QuorumNotReached {
        /// Number of responses required to reach quorum
        required: usize,
        /// Number of responses actually received
        received: usize,
    },

    /// Error indicating that a digest mismatch occurred, with expected and actual digest values.
    #[error("Digest mismatch: expected {expected:?}, got {actual:?}")]
    DigestMismatch {
        /// Expected digest value
        expected: Box<[u8]>,
        /// Actual digest value
        actual: Box<[u8]>,
    },

    /// Error indicating that an unsupported feature was encountered, with a message describing the unsupported feature.
    #[error("Unsupported feature: {0}")]
    Unsupported(&'static str),
    /// Error indicating that an unsupported chain ID was encountered.
    #[error("Unsupported chain ID: {0}")]
    UnsupportedChain(ChainId),
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(Box::new(value))
    }
}

impl From<http::Error> for Error {
    fn from(value: http::Error) -> Self {
        Self::Http(Box::new(value))
    }
}
