//! WASM bindings for UTS Core library.

use uts_core::codec::{Decode, Encode, v1::Timestamp};
use wasm_bindgen::prelude::*;

/// This assumes `timestamps` is an array of serialized Timestamp byte arrays.
#[wasm_bindgen]
pub fn merge_timestamps(timestamps: JsValue) -> Result<Vec<u8>, JsError> {
    let timestamps: Vec<Vec<u8>> = serde_wasm_bindgen::from_value(timestamps)?;
    let timestamps: Vec<Timestamp> = timestamps
        .into_iter()
        .map(|data| {
            let mut decoder = &data[..];
            Timestamp::decode(&mut decoder)
                .map_err(|e| JsError::new(&format!("Decode error: {}", e)))
        })
        .collect::<Result<_, _>>()?;

    let merged =
        Timestamp::try_merge(timestamps).map_err(|e| JsError::new(&format!("Error: {}", e)))?;

    let mut encoded = Vec::new();
    merged
        .encode(&mut encoded)
        .map_err(|e| JsError::new(&format!("Encode error: {}", e)))?;
    Ok(encoded)
}
