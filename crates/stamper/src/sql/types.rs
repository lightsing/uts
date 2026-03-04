use alloy_primitives::{FixedBytes, hex};
use sqlx::{
    Encode, Sqlite, Type,
    encode::IsNull,
    error::BoxDynError,
    sqlite::{SqliteArgumentValue, SqliteTypeInfo},
};

macro_rules! define_enum {
    ($ty:ty) => {
        impl ::sqlx::Type<sqlx::Sqlite> for $ty {
            fn type_info() -> ::sqlx::sqlite::SqliteTypeInfo {
                <String as ::sqlx::Type<::sqlx::Sqlite>>::type_info()
            }

            fn compatible(ty: &::sqlx::sqlite::SqliteTypeInfo) -> bool {
                ::sqlx::TypeInfo::name(ty) == "TEXT"
            }
        }

        impl<'q> ::sqlx::Encode<'q, ::sqlx::Sqlite> for $ty {
            fn encode_by_ref(
                &self,
                args: &mut Vec<::sqlx::sqlite::SqliteArgumentValue<'q>>,
            ) -> Result<::sqlx::encode::IsNull, ::sqlx::error::BoxDynError> {
                let s: &'static str = self.into();
                args.push(::sqlx::sqlite::SqliteArgumentValue::Text(s.into()));

                Ok(::sqlx::encode::IsNull::No)
            }
        }

        impl<'r> ::sqlx::Decode<'r, sqlx::Sqlite> for $ty {
            fn decode(
                value: ::sqlx::sqlite::SqliteValueRef<'r>,
            ) -> Result<Self, ::sqlx::error::BoxDynError> {
                let text = <&str as ::sqlx::Decode<::sqlx::Sqlite>>::decode(value)?;
                ::core::str::FromStr::from_str(text)
                    .map_err(|e| Box::new(e) as ::sqlx::error::BoxDynError)
            }
        }
    };
}

define_enum!(super::AttestationResult);

#[derive(Debug)]
pub struct Wrapper<T>(pub T);

impl<const N: usize> Type<Sqlite> for Wrapper<FixedBytes<N>> {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <String as Type<Sqlite>>::compatible(ty)
    }
}

impl<const N: usize> Encode<'_, Sqlite> for Wrapper<FixedBytes<N>> {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'_>>,
    ) -> Result<IsNull, BoxDynError> {
        let s = hex::encode(self.0.as_slice());
        args.push(SqliteArgumentValue::Text(s.into()));
        Ok(IsNull::No)
    }
}

impl<'r, const N: usize> ::sqlx::Decode<'r, Sqlite> for Wrapper<FixedBytes<N>> {
    fn decode(value: ::sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let text = <&str as ::sqlx::Decode<Sqlite>>::decode(value)?;
        let bytes = hex::decode(text)?;
        if bytes.len() != N {
            return Err(format!("expected {N} bytes but got {}", bytes.len()).into());
        }
        let mut ret = [0u8; N];
        ret.copy_from_slice(&bytes);
        Ok(Wrapper(ret.into()))
    }
}
