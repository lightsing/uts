use crate::TextWrapper;
use alloy_primitives::{Address, FixedBytes, Uint, hex};
use sqlx::{
    Encode, Sqlite, Type,
    encode::IsNull,
    error::BoxDynError,
    sqlite::{SqliteArgumentValue, SqliteTypeInfo},
};

impl<const N: usize> Type<Sqlite> for TextWrapper<FixedBytes<N>> {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <String as Type<Sqlite>>::compatible(ty)
    }
}

impl<const N: usize> Encode<'_, Sqlite> for TextWrapper<FixedBytes<N>> {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'_>>,
    ) -> Result<IsNull, BoxDynError> {
        let s = hex::encode(self.0.as_slice());
        args.push(SqliteArgumentValue::Text(s.into()));
        Ok(IsNull::No)
    }
}

impl<'r, const N: usize> ::sqlx::Decode<'r, Sqlite> for TextWrapper<FixedBytes<N>> {
    fn decode(value: ::sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let text = <&str as ::sqlx::Decode<Sqlite>>::decode(value)?;
        let bytes = hex::decode(text)?;
        if bytes.len() != N {
            return Err(format!("expected {N} bytes but got {}", bytes.len()).into());
        }
        let mut ret = [0u8; N];
        ret.copy_from_slice(&bytes);
        Ok(TextWrapper(ret.into()))
    }
}

impl<const BITS: usize, const LIMBS: usize> Type<Sqlite> for TextWrapper<Uint<BITS, LIMBS>> {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <String as Type<Sqlite>>::compatible(ty)
    }
}

impl<const BITS: usize, const LIMBS: usize> Encode<'_, Sqlite> for TextWrapper<Uint<BITS, LIMBS>> {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'_>>,
    ) -> Result<IsNull, BoxDynError> {
        let s = self.0.to_string();
        args.push(SqliteArgumentValue::Text(s.into()));
        Ok(IsNull::No)
    }
}

impl<'r, const BITS: usize, const LIMBS: usize> ::sqlx::Decode<'r, Sqlite>
    for TextWrapper<Uint<BITS, LIMBS>>
{
    fn decode(value: ::sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let text = <&str as ::sqlx::Decode<Sqlite>>::decode(value)?;
        let uint = text.parse::<Uint<BITS, LIMBS>>()?;
        Ok(TextWrapper(uint))
    }
}

impl Type<Sqlite> for TextWrapper<Address> {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <String as Type<Sqlite>>::compatible(ty)
    }
}

impl Encode<'_, Sqlite> for TextWrapper<Address> {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'_>>,
    ) -> Result<IsNull, BoxDynError> {
        TextWrapper(self.0.0).encode_by_ref(args)
    }
}

impl<'r> ::sqlx::Decode<'r, Sqlite> for TextWrapper<Address> {
    fn decode(value: ::sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let text_wrapper = <TextWrapper<FixedBytes<20>> as ::sqlx::Decode<Sqlite>>::decode(value)?;
        Ok(TextWrapper(Address(text_wrapper.0)))
    }
}
