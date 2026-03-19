use crate::{UtsError, codec::v1::Attestation};
use std::sync::Arc;
use uniffi::Object;
use uts_core::{
    alloc::Global,
    codec::{Decode, Encode, v1 as codec_v1},
};

#[derive(Debug, Object)]
pub struct Timestamp {
    inner: codec_v1::Timestamp,
}

#[uniffi::export]
impl Timestamp {
    #[uniffi::constructor]
    pub fn decode(buffer: &[u8]) -> Result<Self, UtsError> {
        let inner = codec_v1::Timestamp::decode(&mut &*buffer)?;

        Ok(Self { inner })
    }

    pub fn encode(&self) -> Result<Vec<u8>, UtsError> {
        let mut buffer = Vec::with_capacity(1024);
        self.inner.encode(&mut buffer)?;
        buffer.shrink_to_fit();

        Ok(buffer)
    }

    #[uniffi::constructor]
    pub fn try_merge_in(timestamps: Vec<Arc<Timestamp>>) -> Result<Self, UtsError> {
        let timestamps = timestamps
            .into_iter()
            .map(|t| t.inner.clone())
            .collect::<uts_core::alloc::vec::Vec<_>>();

        let inner = codec_v1::Timestamp::try_merge_in(timestamps, Global)?;
        Ok(Self { inner })
    }

    /// Try finalizes the timestamp with the given input data.
    ///
    /// Returns an error if the timestamp is already finalized with different input data.
    pub fn try_finalize(&self, input: &[u8]) -> Result<(), UtsError> {
        self.inner.try_finalize(input)?;
        Ok(())
    }

    pub fn attestations(&self) -> Result<Vec<Attestation>, UtsError> {
        self.inner
            .attestations()
            .map(Attestation::try_from)
            .collect()
    }
}
