//! WASM bindings for UTS Core library.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use serde_with::{hex::Hex, serde_as};
use uts_core::codec::{
    Decode, Encode,
    v1::{
        Attestation, BitcoinAttestation, DetachedTimestamp, DigestHeader, EthereumUTSAttestation,
        MayHaveInput, PendingAttestation, Timestamp, opcode::OpCode,
    },
};
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

/// Pack a detached timestamp with the given digest header.
#[wasm_bindgen]
pub fn pack_detached_timestamp(digest: JsValue, timestamp: Vec<u8>) -> Result<Vec<u8>, JsError> {
    let digest: DigestHeader = serde_wasm_bindgen::from_value(digest)?;
    let mut decoder = &timestamp[..];
    let timestamp = Timestamp::decode(&mut decoder)
        .map_err(|e| JsError::new(&format!("Decode error: {}", e)))?;
    let detached = DetachedTimestamp::try_from_parts(digest, timestamp)
        .map_err(|e| JsError::new(&format!("Error: {}", e)))?;

    let mut encoded = Vec::new();
    detached
        .encode(&mut encoded)
        .map_err(|e| JsError::new(&format!("Encode error: {}", e)))?;
    Ok(encoded)
}

/// Trace the execution steps of a finalized timestamp.
#[wasm_bindgen]
pub fn trace_timestamp(timestamp: Vec<u8>) -> Result<JsValue, JsError> {
    let mut decoder = &timestamp[..];
    let timestamp = Timestamp::decode(&mut decoder)
        .map_err(|e| JsError::new(&format!("Decode error: {}", e)))?;
    if !timestamp.is_finalized() {
        return Err(JsError::new("Can only trace finalized timestamps"));
    }
    serde_wasm_bindgen::to_value(&serialize_chain(&timestamp))
        .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

fn serialize_chain(mut current_node: &Timestamp) -> Value {
    #[serde_as]
    #[derive(Serialize, Deserialize)]
    struct ExecutionStep {
        op: OpCode,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde_as(as = "Option<Hex>")]
        data: Option<Vec<u8>>,
        #[serde_as(as = "Hex")]
        input: Vec<u8>,
        #[serde_as(as = "Hex")]
        output: Vec<u8>,
    }

    #[serde_as]
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    enum AttestationStep {
        Pending {
            url: String,
        },
        Bitcoin {
            height: u32,
        },
        EthereumUTS {
            chain: u64,
            height: u64,
        },
        Unknown {
            #[serde_as(as = "Hex")]
            tag: Vec<u8>,
            #[serde_as(as = "Hex")]
            data: Vec<u8>,
        },
    }
    let mut chain = Vec::new();
    loop {
        match current_node {
            Timestamp::Attestation(raw) => {
                if raw.tag == PendingAttestation::TAG {
                    let pending = PendingAttestation::from_raw(raw).unwrap();
                    chain.push(json!(AttestationStep::Pending {
                        url: pending.uri.to_string(),
                    }));
                } else if raw.tag == BitcoinAttestation::TAG {
                    let btc = BitcoinAttestation::from_raw(raw).unwrap();
                    chain.push(json!(AttestationStep::Bitcoin { height: btc.height }));
                } else if raw.tag == EthereumUTSAttestation::TAG {
                    let eth = EthereumUTSAttestation::from_raw(raw).unwrap();
                    chain.push(json!(AttestationStep::EthereumUTS {
                        chain: eth.chain.id(),
                        height: eth.height,
                    }));
                } else {
                    chain.push(json!(AttestationStep::Unknown {
                        tag: raw.tag.to_vec(),
                        data: raw.data.to_vec(),
                    }));
                }
                break;
            }
            Timestamp::Step(step) => {
                let op = step.op();
                let input = step.input().unwrap().to_vec();
                let output = op.execute(&input, step.data());
                chain.push(
                    serde_json::to_value(&ExecutionStep {
                        op,
                        data: if op.has_immediate() {
                            None
                        } else {
                            Some(step.data().to_vec())
                        },
                        input,
                        output,
                    })
                    .unwrap(),
                );

                let next = step.next();
                match next.len() {
                    0 => break,
                    1 => current_node = &next[0],
                    _ => {
                        let forks: Vec<Value> =
                            next.iter().map(|child| serialize_chain(child)).collect();
                        chain.push(Value::Array(forks));
                        break;
                    }
                }
            }
        }
    }

    Value::Array(chain)
}
