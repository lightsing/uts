use crate::{
    codec::{Decode, Encode},
    error::EncodeError,
};

macro_rules! leb128 {
    ($ty:ty) => {
        paste::paste! {
            impl super::Encode for $ty {
                #[inline]
                fn encode(&self, mut encoder: impl crate::codec::Encoder) -> Result<(), $crate::error::EncodeError> {
                    let mut n = *self;
                    let mut buf = [0u8; <$ty>::BITS.div_ceil(7) as usize];
                    let mut i = 0;

                    loop {
                        let mut byte = (n as u8) & 0x7f;
                        n >>= 7;

                        if n != 0 {
                            byte |= 0x80;
                        }

                        buf[i] = byte;
                        i += 1;

                        if n == 0 {
                            break;
                        }
                    }

                    encoder.write_all(&buf[..i])?;
                    Ok(())
                }
            }

            impl super::Decode for $ty {
                #[inline]
                fn decode(mut decoder: impl crate::codec::Decoder) -> Result<Self, $crate::error::DecodeError> {
                    let mut ret: $ty = 0;
                    let mut shift: u32 = 0;

                    loop {
                        // Bottom 7 bits are value bits
                        let byte = decoder.decode_byte()?;
                        ret |= ((byte & 0x7f) as $ty)
                            .shl_exact(shift)
                            .ok_or($crate::error::DecodeError::LEB128Overflow(<$ty>::BITS))?;
                        // Top bit is a continue bit
                        if byte & 0x80 == 0 {
                            break;
                        }
                        shift = shift
                            .checked_add(7)
                            .ok_or($crate::error::DecodeError::LEB128Overflow(<$ty>::BITS))?;
                    }

                    Ok(ret)
                }
            }
        }
    };
    ($($ty:ty),+ $(,)?) => {
        $(leb128!($ty);)+
    };
}

leb128!(u16, u32, u64, u128);

impl Encode for u8 {
    #[inline]
    fn encode(&self, mut encoder: impl crate::codec::Encoder) -> Result<(), EncodeError> {
        encoder.write_all(&[*self])?;
        Ok(())
    }
}

impl Decode for u8 {
    #[inline]
    fn decode(mut decoder: impl crate::codec::Decoder) -> Result<Self, crate::error::DecodeError> {
        let mut byte = [0u8; 1];
        decoder.read_exact(&mut byte)?;
        Ok(byte[0])
    }
}

impl Encode for usize {
    #[inline]
    fn encode(&self, mut encoder: impl crate::codec::Encoder) -> Result<(), EncodeError> {
        let val: u32 = (*self).try_into().map_err(|_| EncodeError::UsizeOverflow)?;
        val.encode(&mut encoder)?;
        Ok(())
    }
}

impl Decode for usize {
    #[inline]
    fn decode(mut decoder: impl crate::codec::Decoder) -> Result<Self, crate::error::DecodeError> {
        let val = u32::decode(&mut decoder)?;
        Ok(val as usize)
    }
}
