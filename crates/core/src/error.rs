use crate::codec::v1::opcode::OpCode;

/// Errors returned while decoding proofs.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    /// File began with invalid magic bytes.
    #[error("bad magic bytes")]
    BadMagic,
    /// File has a version we do not understand.
    #[error("bad version")]
    BadVersion,
    /// Expected an attestation tag but decoded something else.
    #[error("bad attestation tag")]
    BadAttestationTag,
    /// Read an LEB128-encoded integer that overflowed the expected size.
    #[error("read a LEB128 value overflows {0} bits")]
    LEB128Overflow(u32),
    /// Encountered an unrecognized opcode.
    #[error("unrecognized opcode: 0x{0:02x}")]
    BadOpCode(u8),
    /// Expected a digest opcode but decoded something else.
    #[error("expected digest opcode but got: {0}")]
    ExpectedDigestOp(OpCode),
    /// Read a value that is not in the allowed range.
    #[error("read value out of range")]
    OutOfRange,
    /// Encountered an invalid character in a URI.
    #[error("invalid character in URI")]
    InvalidUriChar,
    /// URI is too long.
    #[error("URI too long")]
    UriTooLong,
    /// Recursed deeper than allowed while decoding the proof.
    #[error("recursion limit reached")]
    RecursionLimit,
    /// Reached end-of-file unexpectedly.
    #[error("unexpected end of file")]
    UnexpectedEof,
    /// General I/O error
    #[cfg(feature = "std")]
    #[error("I/O error: {0}")]
    Io(std::io::Error),
}

/// Errors returned while encoding proofs.
#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    /// Tried to encode a `usize` exceeding `u32::MAX`.
    #[error("tried to encode a usize exceeding u32::MAX")]
    UsizeOverflow,
    /// Encountered an invalid character in a URI.
    #[error("invalid character in URI")]
    InvalidUriChar,
    /// URI is too long.
    #[error("URI too long")]
    UriTooLong,
    /// General I/O error
    #[cfg(feature = "std")]
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(feature = "std")]
impl From<std::io::Error> for DecodeError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::UnexpectedEof => Self::UnexpectedEof,
            _ => Self::Io(err),
        }
    }
}
