use crate::UtsError;
use alloy_primitives::B256;
use uniffi::custom_type;
use uts_core::codec::v1::AttestationTag;

custom_type!(B256, Vec<u8>, {
    remote,
    lower: |value| value.to_vec(),
    try_lift: |bytes| {
        Ok(B256::new(bytes.try_into().map_err(|_| UtsError::InvalidFixedBytes)?))
    }
});

// very sad here we can't use a macro to call `custom_type`

custom_type!(AttestationTag, Vec<u8>, {
    remote,
    lower: |value| value.to_vec(),
    try_lift: |bytes| {
        Ok(AttestationTag::new(bytes.try_into().map_err(|_| UtsError::InvalidFixedBytes)?))
    }
});
