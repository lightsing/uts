use std::{fmt::Debug, sync::PoisonError};
use tracing::error;
use uts_core::{
    codec::v1::FinalizationError,
    error::{DecodeError, EncodeError},
};

/// Exported Error type
#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum UtsError {
    /// An error occurred while decoding a proof.
    #[error("decode error: {0}")]
    DecodeError(String),
    /// An error occurred while encoding a proof.
    #[error("encode error: {0}")]
    EncodeError(String),

    /// Invaild length of a fixed bytes type
    #[error("invalid length of a fixed bytes type")]
    InvalidFixedBytes,

    #[error("unknown digest operation")]
    UnknownDigestOp,
    #[error("invalid digest length")]
    InvalidDigestLength,
    #[error("unknown operation")]
    UnknownOp,

    /// An error occurred when finalization of a timestamp failed due to conflicting inputs.
    #[error("failed to finalize timestamp due to conflicting inputs")]
    FinalizationError,

    /// The Mutex/RwLock got poisoned
    #[error("the Mutex/RwLock got poisoned")]
    Poisoned,
    /// An error occurred when bug happens, this means some precondition fails.
    #[error("an unexpected error occurred: {0}")]
    Unexpected(&'static str),
}

pub(crate) trait UtsErrorExt<T> {
    fn infallible(self, ctx: &'static str) -> Result<T, UtsError>;
}

impl<T, E: Debug> UtsErrorExt<T> for Result<T, E> {
    fn infallible(self, ctx: &'static str) -> Result<T, UtsError> {
        self.inspect_err(|e| error!("unexpected error occurred in {}: {:?}", ctx, e))
            .map_err(|_| UtsError::Unexpected(ctx))
    }
}

impl<T> UtsErrorExt<T> for Option<T> {
    fn infallible(self, ctx: &'static str) -> Result<T, UtsError> {
        self.ok_or_else(|| {
            error!("unexpected error occurred in {}: expect Some but None", ctx);
            UtsError::Unexpected(ctx)
        })
    }
}

impl From<DecodeError> for UtsError {
    fn from(e: DecodeError) -> Self {
        UtsError::DecodeError(e.to_string())
    }
}

impl From<EncodeError> for UtsError {
    fn from(e: EncodeError) -> Self {
        UtsError::EncodeError(e.to_string())
    }
}

impl From<FinalizationError> for UtsError {
    fn from(_: FinalizationError) -> Self {
        UtsError::FinalizationError
    }
}

impl<T> From<PoisonError<T>> for UtsError {
    fn from(_: PoisonError<T>) -> Self {
        error!("trying acquire a lock is poisoned");
        UtsError::Poisoned
    }
}
