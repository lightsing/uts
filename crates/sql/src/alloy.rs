use crate::TextWrapper;
use alloy_primitives::{FixedBytes, hex};
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
