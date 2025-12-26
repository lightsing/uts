use crate::types::Wrapped;
use alloy_primitives::FixedBytes;
use sea_orm::{
    ColumnType, QueryResult, TryGetError, TryGetable, Value,
    sea_query::{ArrayType, ValueType, ValueTypeErr},
};
use std::{any::type_name, sync::Arc};

impl<const N: usize> From<Wrapped<FixedBytes<N>>> for Value {
    fn from(x: Wrapped<FixedBytes<N>>) -> Self {
        Value::Bytes(Some(x.to_vec()))
    }
}

impl<const N: usize> ValueType for Wrapped<FixedBytes<N>> {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::Bytes(Some(bytes)) => {
                let inner = FixedBytes::try_from(&*bytes).map_err(|_| ValueTypeErr)?;
                Ok(Wrapped(inner))
            }
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        format!("FixedBytes<{N}>")
    }

    fn array_type() -> ArrayType {
        ArrayType::Bytes
    }

    fn column_type() -> ColumnType {
        ColumnType::Binary(N as u32)
    }
}

impl<const N: usize> TryGetable for Wrapped<FixedBytes<N>> {
    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        let bytes: Vec<u8> = res.try_get_by(index)?;
        let fixed = FixedBytes::try_from(&*bytes).map_err(|e| {
            TryGetError::DbErr(sea_orm::DbErr::TryIntoErr {
                from: type_name::<Vec<u8>>(),
                into: type_name::<FixedBytes<N>>(),
                source: Arc::new(e),
            })
        })?;
        Ok(Wrapped(fixed))
    }
}
