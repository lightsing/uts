//! # OpenTimestamps OpCodes
//!
//! It contains opcode information and utilities to work with opcodes.

use crate::{
    codec::{Decode, DecodeIn, Decoder, Encode, Encoder},
    error::{DecodeError, EncodeError},
};
use alloc::{alloc::Allocator, vec::Vec};
use alloy_primitives::hex;
use core::{fmt, hint::unreachable_unchecked};
use digest::Digest;
use ripemd::Ripemd160;
use sha1::Sha1;
use sha2::Sha256;
use sha3::Keccak256;

/// An OpenTimestamps opcode.
///
/// This is always a valid opcode.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)
)]
#[repr(transparent)]
pub struct OpCode(u8);

impl fmt::Debug for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl fmt::Display for OpCode {
    /// Formats the opcode as a string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl Encode for OpCode {
    #[inline]
    fn encode(&self, encoder: &mut impl Encoder) -> Result<(), EncodeError> {
        encoder.encode_byte(self.tag())
    }
}

impl Decode for OpCode {
    #[inline]
    fn decode(decoder: &mut impl Decoder) -> Result<Self, DecodeError> {
        let byte = decoder.decode_byte()?;
        OpCode::new(byte).ok_or(DecodeError::BadOpCode(byte))
    }
}

impl OpCode {
    /// Returns the 8-bit tag identifying the opcode.
    #[inline]
    pub const fn tag(self) -> u8 {
        self.0
    }

    /// Returns `true` when the opcode requires an immediate operand.
    #[inline]
    pub const fn has_immediate(&self) -> bool {
        matches!(*self, Self::APPEND | Self::PREPEND)
    }

    /// Returns `true` for control opcodes.
    #[inline]
    pub const fn is_control(&self) -> bool {
        matches!(*self, Self::ATTESTATION | Self::FORK)
    }

    /// Returns `true` for digest opcodes.
    #[inline]
    pub const fn is_digest(&self) -> bool {
        self.as_digest().is_some()
    }

    /// Returns the digest opcode wrapper, if applicable.
    #[inline]
    pub const fn as_digest(&self) -> Option<DigestOp> {
        match *self {
            Self::SHA1 | Self::SHA256 | Self::RIPEMD160 | Self::KECCAK256 => Some(DigestOp(*self)),
            _ => None,
        }
    }

    /// Executes the opcode on the given input data, with an optional immediate value.
    ///
    /// # Panics
    ///
    /// Panics if the opcode is a control opcode.
    #[inline]
    pub fn execute(&self, input: impl AsRef<[u8]>, immediate: impl AsRef<[u8]>) -> Vec<u8> {
        self.execute_in(input, immediate, alloc::alloc::Global)
    }

    /// Executes the opcode on the given input data, with an optional immediate value.
    ///
    /// # Panics
    ///
    /// Panics if the opcode is a control opcode.
    #[inline]
    pub fn execute_in<A: Allocator>(
        &self,
        input: impl AsRef<[u8]>,
        immediate: impl AsRef<[u8]>,
        alloc: A,
    ) -> Vec<u8, A> {
        if let Some(digest_op) = self.as_digest() {
            return digest_op.execute_in(input, alloc);
        }

        let input = input.as_ref();
        match *self {
            Self::APPEND => {
                let immediate = immediate.as_ref();
                let mut out = Vec::with_capacity_in(input.len() + immediate.len(), alloc);
                out.extend_from_slice(input);
                out.extend_from_slice(immediate);
                out
            }
            Self::PREPEND => {
                let immediate = immediate.as_ref();
                let mut out = Vec::with_capacity_in(input.len() + immediate.len(), alloc);
                out.extend_from_slice(immediate);
                out.extend_from_slice(input);
                out
            }
            Self::REVERSE => {
                let len = input.len();
                let mut out = Vec::<u8, A>::with_capacity_in(len, alloc);

                unsafe {
                    // SAFETY: The vector capacity is set to len, so setting the length to len is valid.
                    out.set_len(len);

                    // LLVM will take care of vectorization here.
                    let input_ptr = input.as_ptr();
                    let out_ptr = out.as_mut_ptr();
                    for i in 0..len {
                        // SAFETY: both pointers are valid for `len` bytes.
                        *out_ptr.add(i) = *input_ptr.add(len - 1 - i);
                    }
                }
                out
            }
            Self::HEXLIFY => {
                let hex_len = input.len() * 2;
                let mut out = Vec::<u8, A>::with_capacity_in(hex_len, alloc);
                // SAFETY: that the vector is actually the specified size.
                unsafe {
                    out.set_len(hex_len);
                }
                // SAFETY: the output buffer is large enough.
                unsafe {
                    hex::encode_to_slice(input, &mut out).unwrap_unchecked();
                }
                out
            }
            _ => panic!("Cannot execute control opcode"),
        }
    }
}

impl PartialEq<u8> for OpCode {
    fn eq(&self, other: &u8) -> bool {
        self.tag().eq(other)
    }
}

/// An OpenTimestamps digest opcode.
///
/// This is always a valid opcode.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct DigestOp(OpCode);

impl fmt::Debug for DigestOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.name())
    }
}

impl fmt::Display for DigestOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<OpCode> for DigestOp {
    fn eq(&self, other: &OpCode) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<u8> for DigestOp {
    fn eq(&self, other: &u8) -> bool {
        self.0.eq(other)
    }
}

impl Encode for DigestOp {
    #[inline]
    fn encode(&self, encoder: &mut impl Encoder) -> Result<(), EncodeError> {
        self.0.encode(encoder)
    }
}

impl<A: Allocator> DecodeIn<A> for DigestOp {
    #[inline]
    fn decode_in(decoder: &mut impl Decoder, _alloc: A) -> Result<Self, DecodeError> {
        let opcode = OpCode::decode(decoder)?;
        opcode
            .as_digest()
            .ok_or(DecodeError::ExpectedDigestOp(opcode))
    }
}

impl DigestOp {
    /// Returns the wrapped opcode.
    #[inline]
    pub const fn to_opcode(self) -> OpCode {
        self.0
    }

    /// Returns the 8-bit tag identifying the digest opcode.
    #[inline]
    pub const fn tag(&self) -> u8 {
        self.0.tag()
    }
}

/// Extension trait for `Digest` implementors to get the corresponding `DigestOp`.
pub trait DigestOpExt: Digest {
    const OPCODE: DigestOp;

    fn opcode() -> DigestOp;
}

macro_rules! define_opcodes {
    ($($val:literal => $variant:ident),* $(,)?) => {
         $(
            #[doc = concat!("The `", stringify!($val), "` (\"", stringify!($variant),"\") opcode.")]
            pub const $variant: u8 = $val;
        )*

        $(
            impl OpCode {
                #[doc = concat!("The `", stringify!($val), "` (\"", stringify!($variant),"\") opcode.")]
                pub const $variant: Self = Self($val);
            }
        )*

        impl OpCode {
            #[inline]
            pub const fn new(v: u8) -> Option<Self> {
                match v {
                    $( $val => Some(Self::$variant), )*
                    _ => None,
                }
            }

            #[inline]
            pub const fn name(&self) -> &'static str {
                match *self {
                    $( Self::$variant => stringify!($variant), )*
                    // SAFETY: unreachable as all variants are covered.
                    _ => unsafe { unreachable_unchecked() }
                }
            }
        }

        /// Error returned when parsing an invalid opcode from a string.
        #[derive(Debug)]
        pub struct OpCodeFromStrError;

        impl core::fmt::Display for OpCodeFromStrError {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "invalid opcode string")
            }
        }

        #[cfg(feature = "std")]
        impl std::error::Error for OpCodeFromStrError {}

        impl core::str::FromStr for OpCode {
            type Err = OpCodeFromStrError;

            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $( stringify!($variant) => Ok(Self::$variant), )*
                    _ => Err(OpCodeFromStrError),
                }
            }
         }
    };
}

macro_rules! define_digest_opcodes {
    ($($val:literal => $variant:ident),* $(,)?) => {
        $(
            impl DigestOp {
                #[doc = concat!("The `", stringify!($val), "` (\"", stringify!($variant),"\") digest opcode.")]
                pub const $variant: Self = Self(OpCode::$variant);
            }
        )*

        impl DigestOp {
            /// Returns the output length of the digest in bytes.
            #[inline]
            pub const fn output_size(&self) -> usize {
                use digest::typenum::Unsigned;
                paste::paste! {
                    match *self {
                        $( Self::$variant => <[<$variant:camel>] as ::digest::OutputSizeUser>::OutputSize::USIZE, )*
                        // SAFETY: unreachable as all variants are covered.
                        _ => unsafe { unreachable_unchecked() }
                    }
                }
            }

            /// Executes the digest operation on the input data.
            pub fn execute(&self, input: impl AsRef<[u8]>) -> ::alloc::vec::Vec<u8> {
                self.execute_in(input, ::alloc::alloc::Global)
            }

            /// Executes the digest operation on the input data.
            pub fn execute_in<A: Allocator>(&self, input: impl AsRef<[u8]>, alloc: A) -> ::alloc::vec::Vec<u8, A> {
                match *self {
                    $( Self::$variant => {
                        paste::paste! {
                            let mut hasher = [<$variant:camel>]::new();
                            hasher.update(input);
                            hasher.finalize().to_vec_in(alloc)
                        }
                    }, )*
                    // SAFETY: unreachable as all variants are covered.
                    _ => unsafe { unreachable_unchecked() }
                }
            }
        }
        paste::paste! {
            $(
                impl DigestOpExt for [<$variant:camel>] {
                    const OPCODE: DigestOp = DigestOp::$variant;

                    #[inline]
                    fn opcode() -> DigestOp {
                        DigestOp::$variant
                    }
                }
            )*
        }
    };
}

macro_rules! impl_simple_step {
    ($variant:ident) => {paste::paste! {
        impl<A: Allocator + Clone> $crate::codec::v1::timestamp::builder::TimestampBuilder<A> {
            #[doc = concat!("Push the `", stringify!($variant), "` opcode.")]
            pub fn [< $variant:lower >](&mut self) -> &mut Self {
                self.push_step(OpCode::[<$variant>])
            }
        }
    }};
    ($($variant:ident),* $(,)?) => {
        $(
            impl_simple_step! { $variant }
        )*
    };
}

macro_rules! impl_step_with_data {
    ($variant:ident) => {paste::paste! {
        impl<A: Allocator + Clone> $crate::codec::v1::timestamp::builder::TimestampBuilder<A> {
            #[doc = concat!("Push the `", stringify!($variant), "` opcode.")]
            pub fn [< $variant:lower >](&mut self, data: ::alloc::vec::Vec<u8, A>) -> &mut Self {
                self.push_immediate_step(OpCode::[<$variant>], data)
            }
        }
    }};
    ($($variant:ident),* $(,)?) => {
        $(
            impl_step_with_data! { $variant }
        )*
    };
}

define_opcodes! {
    0x02 => SHA1,
    0x03 => RIPEMD160,
    0x08 => SHA256,
    0x67 => KECCAK256,
    0xf0 => APPEND,
    0xf1 => PREPEND,
    0xf2 => REVERSE,
    0xf3 => HEXLIFY,
    0x00 => ATTESTATION,
    0xff => FORK,
}

define_digest_opcodes! {
    0x02 => SHA1,
    0x03 => RIPEMD160,
    0x08 => SHA256,
    0x67 => KECCAK256,
}

impl_simple_step! {
    SHA1,
    RIPEMD160,
    SHA256,
    KECCAK256,
    REVERSE,
    HEXLIFY,
}

impl_step_with_data! {
    APPEND,
    PREPEND,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_len() {
        assert_eq!(DigestOp::SHA1.output_size(), 20);
        assert_eq!(DigestOp::RIPEMD160.output_size(), 20);
        assert_eq!(DigestOp::SHA256.output_size(), 32);
        assert_eq!(DigestOp::KECCAK256.output_size(), 32);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_opcode() {
        let opcode = OpCode::SHA256;
        let serialized = serde_json::to_string(&opcode).unwrap();
        assert_eq!(serialized, "\"SHA256\"");
        let deserialized: OpCode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, opcode);

        let digest_op = DigestOp::SHA256;
        let serialized = serde_json::to_string(&digest_op).unwrap();
        assert_eq!(serialized, "\"SHA256\"");
        let deserialized: DigestOp = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, digest_op);
    }
}
