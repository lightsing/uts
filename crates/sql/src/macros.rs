/// A helper macro to define a text-based enum that can be stored in a SQLite database.
/// The enum must implement `Into<&'static str>` and `FromStr`.
#[macro_export]
macro_rules! define_text_enum {
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

/// A helper macro to define a `migrate` function that runs database migrations from a specified directory.
#[macro_export]
macro_rules! migrator {
    ($dir:literal) => {
        /// Run database migrations. This should be run at the start of the application to ensure the
        /// database schema is up to date.
        #[inline]
        pub async fn migrate<'a, A>(migrator: A) -> Result<(), ::sqlx::migrate::MigrateError>
        where
            A: ::sqlx::Acquire<'a>,
            <A::Connection as ::core::ops::Deref>::Target: ::sqlx::migrate::Migrate,
        {
            ::sqlx::migrate!($dir).run(migrator).await
        }
    };
    () => {
        migrator!("./migrations");
    };
}
