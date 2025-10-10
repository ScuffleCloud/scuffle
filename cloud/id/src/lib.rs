use std::fmt::Debug;

pub mod exports {
    pub use {diesel, serde, ulid, uuid};
}

#[macro_export]
macro_rules! impl_id {
    ($vis:vis $type:ident, $prefix:literal) => {
        #[derive(
            $crate::exports::diesel::deserialize::FromSqlRow,
            $crate::exports::diesel::expression::AsExpression,
            Debug,
            PartialEq,
            Eq,
            Hash,
            Clone,
            Copy,
            Default,
        )]
        #[diesel(sql_type = $crate::exports::diesel::sql_types::Uuid)]
        $vis struct $type($crate::exports::ulid::Ulid);

        impl ::std::convert::From<$type> for $crate::exports::ulid::Ulid {
            fn from(value: $type) -> Self {
                value.0
            }
        }

        impl ::std::convert::From<$crate::exports::ulid::Ulid> for $type {
            fn from(value: $crate::exports::ulid::Ulid) -> Self {
                Self(value)
            }
        }

        impl ::std::convert::From<$crate::exports::uuid::Uuid> for $type {
            fn from(value: $crate::exports::uuid::Uuid) -> Self {
                Self($crate::exports::ulid::Ulid::from(value))
            }
        }

        impl $crate::exports::diesel::deserialize::FromSql<$crate::exports::diesel::sql_types::Uuid, $crate::exports::diesel::pg::Pg> for $type {
            fn from_sql(bytes: $crate::exports::diesel::pg::PgValue<'_>) -> $crate::exports::diesel::deserialize::Result<Self> {
                let uuid: $crate::exports::uuid::Uuid = $crate::exports::diesel::deserialize::FromSql::from_sql(bytes)?;

                Ok(Self($crate::exports::ulid::Ulid::from(uuid)))
            }
        }

        impl $crate::exports::diesel::serialize::ToSql<$crate::exports::diesel::sql_types::Uuid, $crate::exports::diesel::pg::Pg> for $type {
            fn to_sql<'b>(&'b self, out: &mut $crate::exports::diesel::serialize::Output<'b, '_, $crate::exports::diesel::pg::Pg>) -> $crate::exports::diesel::serialize::Result {
                ::std::io::Write::write_all(out, &self.0.to_bytes())
                    .map(|_| $crate::exports::diesel::serialize::IsNull::No)
                    .map_err(Into::into)
            }
        }

        impl $type {
            $vis const PREFIX: &'static str = $prefix;

            $vis fn new() -> Self {
                Self($crate::exports::ulid::Ulid::new())
            }

            $vis fn ulid(&self) -> $crate::exports::ulid::Ulid {
                self.0
            }

            $vis fn from_ulid(ulid: $crate::exports::ulid::Ulid) -> Self {
                Self(ulid)
            }

            $vis fn datetime(&self) -> ::std::time::SystemTime {
                self.0.datetime()
            }
        }

        impl ::std::str::FromStr for $type {
            type Err = $crate::IdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let id = s.strip_prefix(Self::PREFIX).ok_or($crate::IdParseError::PrefixMismatch(Self::PREFIX))?;
                let id = $crate::exports::ulid::Ulid::from_str(id)?;
                Ok(Self::from_ulid(id))
            }
        }

        impl ::std::fmt::Display for $type {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}{}", Self::PREFIX, self.0)
            }
        }

        impl ::serde::Serialize for $type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let s = ::std::string::String::deserialize(deserializer)?;
                s.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}

#[derive(Debug, thiserror::Error)]
pub enum IdParseError {
    #[error("id doesnt have prefix {0}")]
    PrefixMismatch(&'static str),
    #[error("invalid ulid: {0}")]
    Ulid(#[from] ulid::DecodeError),
}
