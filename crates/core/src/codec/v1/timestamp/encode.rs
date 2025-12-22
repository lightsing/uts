use super::*;
use crate::{
    codec::{Encode, Encoder},
    error::EncodeError,
};

impl Encode for Timestamp {
    fn encode(&self, encoder: &mut impl Encoder) -> Result<(), EncodeError> {
        match self {
            Self::Attestation(attestation) => {
                encoder.encode(self.op())?;
                attestation.encode(encoder)?;
            }
            Self::Step(step) if step.op == OpCode::FORK => {
                debug_assert!(step.next.len() >= 2, "FORK must have at least two children");
                for child in step.next.iter().take(step.next.len() - 1) {
                    encoder.encode(self.op())?;
                    child.encode(encoder)?;
                }
                // Encode the last child
                step.next.last().expect("infallible").encode(encoder)?;
            }
            Self::Step(step) => {
                encoder.encode(self.op())?;
                if !step.data.is_empty() {
                    debug_assert!(step.op.has_immediate());
                    encoder.encode_bytes(&step.data)?;
                }
                debug_assert_eq!(step.next.len(), 1);
                step.next[0].encode(encoder)?;
            }
        }

        Ok(())
    }
}
