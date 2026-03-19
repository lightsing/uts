use crate::{UtsError, codec::v1::DigestHeader};
use std::sync::RwLock;
use uniffi::Object;
use uts_core::codec::{Decode, Encode, v1 as codec_v1};

#[derive(Debug, Object)]
pub struct DetachedTimestamp {
    inner: RwLock<codec_v1::DetachedTimestamp>,
}

#[uniffi::export]
impl DetachedTimestamp {
    #[uniffi::constructor]
    pub fn decode(buffer: &[u8]) -> Result<Self, UtsError> {
        let inner = codec_v1::DetachedTimestamp::decode(&mut &*buffer)?;

        Ok(Self {
            inner: RwLock::new(inner),
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, UtsError> {
        let mut buffer = Vec::with_capacity(1024);
        self.inner.read()?.encode(&mut buffer)?;
        buffer.shrink_to_fit();

        Ok(buffer)
    }

    /// Tries to finalize the detached timestamp's timestamp with the header's digest.
    ///
    /// Returns an error if the timestamp cannot be finalized.
    pub fn try_finalize(&self) -> Result<(), UtsError> {
        self.inner.read()?.try_finalize()?;
        Ok(())
    }

    pub fn header(&self) -> Result<DigestHeader, UtsError> {
        self.inner.read()?.header().try_into()
    }
}
