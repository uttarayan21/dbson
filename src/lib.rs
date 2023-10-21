use serde::{Deserialize, Serialize};

/// A wrapper type for BSON data.
///
/// Any type that implements serde::Deserialize && serde::Serialize can be wrapped by this type.
/// and used inside of a database as a blob.
///
/// This type is useful for storing data that is not easily represented in a relational database.
/// For example, a HashMap or a Vec of structs.
/// Currently only supports rusqlite and sqlx
///
/// It's basically a newtype wrapper over T
/// So it implements many of the same traits as T
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct DBson<T>(T);

// impl<'de> Deserialize<'de> for DBson<bson::Document> {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         bson::Document::deserialize(deserializer).map(Self)
//     }
// }

// impl Serialize for DBson<bson::Document> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         self.0.serialize(serializer)
//     }
// }

impl<T> From<T> for DBson<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> DBson<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

#[cfg(feature = "rusqlite")]
#[cfg_attr(docsrs, doc(cfg(feature = "rusqlite")))]
mod impl_rusqlite {
    use rusqlite::{types::FromSql, ToSql};
    impl<T: serde::Serialize> ToSql for super::DBson<T> {
        fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
            let bytes = bson::to_vec(&self.0)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
            Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Blob(bytes),
            ))
        }
    }

    impl<T: for<'de> serde::de::Deserialize<'de>> FromSql for super::DBson<T> {
        fn column_result(
            value: rusqlite::types::ValueRef<'_>,
        ) -> rusqlite::types::FromSqlResult<Self> {
            let bytes = value.as_blob()?;
            let inner = bson::from_slice(bytes)
                .map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))?;
            Ok(Self::new(inner))
        }
    }
}

#[cfg(feature = "sqlx")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqlx")))]
mod impl_sqlx {
    use sqlx::{
        database::{HasArguments, HasValueRef},
        decode::Decode,
        encode::Encode,
        types::Type,
    };

    impl<'a, T: Serialize + serde::de::DeserializeOwned, DB: sqlx::database::Database> Type<DB>
        for DBson<T>
    where
        &'a [u8]: Type<DB>,
    {
        fn type_info() -> DB::TypeInfo {
            <&[u8] as ::sqlx::types::Type<DB>>::type_info()
        }
    }

    impl<'a, T: Serialize + serde::de::DeserializeOwned, DB: sqlx::database::Database>
        Encode<'a, DB> for DBson<T>
    where
        Vec<u8>: Type<DB>,
        Vec<u8>: Encode<'a, DB>,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <DB as HasArguments<'a>>::ArgumentBuffer,
        ) -> sqlx::encode::IsNull {
            let Ok(bytes) = bson::to_vec(&self.inner) else {
                return sqlx::encode::IsNull::Yes;
            };
            <Vec<u8> as Encode<'a, DB>>::encode_by_ref(&bytes, buf)
        }
        fn encode(
            self,
            buf: &mut <DB as HasArguments<'a>>::ArgumentBuffer,
        ) -> sqlx::encode::IsNull {
            let Ok(bytes) = bson::to_vec(&self.inner) else {
                return sqlx::encode::IsNull::Yes;
            };
            <Vec<u8> as Encode<'a, DB>>::encode(bytes, buf)
        }
    }

    impl<'r, T: Serialize + serde::de::DeserializeOwned, DB: sqlx::database::Database>
        Decode<'r, DB> for DBson<T>
    where
        &'r [u8]: Type<DB>,
        &'r [u8]: Decode<'r, DB>,
    {
        fn decode(
            value: <DB as HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
            let bytes = <&[u8] as Decode<'r, DB>>::decode(value)?;
            let inner = bson::from_slice(&bytes)?;
            Ok(Self { inner })
        }
    }
}
