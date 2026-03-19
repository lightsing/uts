use crate::UtsError;
use uniffi::Enum;
use uts_core::codec::v1::opcode as opcode_v1;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Enum)]
#[non_exhaustive]
#[repr(u8)]
pub enum OpCode {
    Attestation = 0x00,
    Sha1 = 0x02,
    Ripemd160 = 0x03,
    Sha256 = 0x08,
    Keccak256 = 0x67,
    Append = 0xf0,
    Prepend = 0xf1,
    Reverse = 0xf2,
    Hexlify = 0xf3,
    Fork = 0xff,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Enum)]
#[non_exhaustive]
#[repr(u8)]
pub enum DigestOp {
    Sha1 = 0x02,
    Ripemd160 = 0x03,
    Sha256 = 0x08,
    Keccak256 = 0x067,
}

#[uniffi::export]
impl OpCode {
    pub fn tag(self) -> u8 {
        opcode_v1::OpCode::from(self).tag()
    }

    pub fn has_immediate(self) -> bool {
        opcode_v1::OpCode::from(self).has_immediate()
    }

    pub fn is_control(self) -> bool {
        opcode_v1::OpCode::from(self).is_control()
    }

    pub fn is_digest(self) -> bool {
        opcode_v1::OpCode::from(self).is_digest()
    }

    pub fn as_digest(self) -> Result<Option<DigestOp>, UtsError> {
        opcode_v1::OpCode::from(self)
            .as_digest()
            .map(DigestOp::try_from)
            .transpose()
    }

    pub fn execute(self, input: &[u8], immediate: &[u8]) -> Vec<u8> {
        Vec::from_iter(opcode_v1::OpCode::from(self).execute(input, immediate))
    }
}

#[uniffi::export]
impl DigestOp {
    pub fn tag(self) -> u8 {
        opcode_v1::DigestOp::from(self).tag()
    }

    pub fn to_opcode(self) -> Result<OpCode, UtsError> {
        OpCode::try_from(opcode_v1::DigestOp::from(self).to_opcode())
    }

    pub fn output_size(self) -> u32 {
        opcode_v1::DigestOp::from(self).output_size() as u32
    }

    pub fn execute(self, input: &[u8]) -> Vec<u8> {
        Vec::from_iter(opcode_v1::DigestOp::from(self).execute(input))
    }
}

impl From<OpCode> for opcode_v1::OpCode {
    fn from(op: OpCode) -> Self {
        match op {
            OpCode::Attestation => opcode_v1::OpCode::ATTESTATION,
            OpCode::Sha1 => opcode_v1::OpCode::SHA1,
            OpCode::Ripemd160 => opcode_v1::OpCode::RIPEMD160,
            OpCode::Sha256 => opcode_v1::OpCode::SHA256,
            OpCode::Keccak256 => opcode_v1::OpCode::KECCAK256,
            OpCode::Append => opcode_v1::OpCode::APPEND,
            OpCode::Prepend => opcode_v1::OpCode::PREPEND,
            OpCode::Reverse => opcode_v1::OpCode::REVERSE,
            OpCode::Hexlify => opcode_v1::OpCode::HEXLIFY,
            OpCode::Fork => opcode_v1::OpCode::FORK,
        }
    }
}

impl From<DigestOp> for opcode_v1::DigestOp {
    fn from(op: DigestOp) -> Self {
        match op {
            DigestOp::Sha1 => opcode_v1::DigestOp::SHA1,
            DigestOp::Ripemd160 => opcode_v1::DigestOp::RIPEMD160,
            DigestOp::Sha256 => opcode_v1::DigestOp::SHA256,
            DigestOp::Keccak256 => opcode_v1::DigestOp::KECCAK256,
        }
    }
}

impl TryFrom<opcode_v1::OpCode> for OpCode {
    type Error = UtsError;

    fn try_from(op: opcode_v1::OpCode) -> Result<Self, Self::Error> {
        match op {
            opcode_v1::OpCode::ATTESTATION => Ok(OpCode::Attestation),
            opcode_v1::OpCode::SHA1 => Ok(OpCode::Sha1),
            opcode_v1::OpCode::RIPEMD160 => Ok(OpCode::Ripemd160),
            opcode_v1::OpCode::SHA256 => Ok(OpCode::Sha256),
            opcode_v1::OpCode::KECCAK256 => Ok(OpCode::Keccak256),
            opcode_v1::OpCode::APPEND => Ok(OpCode::Append),
            opcode_v1::OpCode::PREPEND => Ok(OpCode::Prepend),
            opcode_v1::OpCode::REVERSE => Ok(OpCode::Reverse),
            opcode_v1::OpCode::HEXLIFY => Ok(OpCode::Hexlify),
            opcode_v1::OpCode::FORK => Ok(OpCode::Fork),
            _ => Err(UtsError::Unexpected("Uncovered OpCode variant")),
        }
    }
}

impl TryFrom<opcode_v1::DigestOp> for DigestOp {
    type Error = UtsError;

    fn try_from(op: opcode_v1::DigestOp) -> Result<Self, Self::Error> {
        match op {
            opcode_v1::DigestOp::SHA1 => Ok(DigestOp::Sha1),
            opcode_v1::DigestOp::RIPEMD160 => Ok(DigestOp::Ripemd160),
            opcode_v1::DigestOp::SHA256 => Ok(DigestOp::Sha256),
            opcode_v1::DigestOp::KECCAK256 => Ok(DigestOp::Keccak256),
            _ => Err(UtsError::Unexpected("Uncovered DigestOp variant")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Uniffi only allows discriminant values to be a literal, so we need to test that all OpCode
    /// variants are covered by the OpCode enum
    #[test]
    fn test_completeness() {
        for tag in 0..u8::MAX {
            let Some(op) = opcode_v1::OpCode::new(tag) else {
                continue;
            };
            let ffi_op = OpCode::try_from(op).expect("missing OpCode variant");
            assert_eq!(ffi_op as u8, tag);
            let Some(digest_op) = op.as_digest() else {
                continue;
            };
            DigestOp::try_from(digest_op).expect("missing DigestOp variant");
            assert_eq!(ffi_op as u8, digest_op.tag());
        }
    }
}
