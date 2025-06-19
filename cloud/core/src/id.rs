use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::io::Write;
use std::ops::Deref;

use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::serialize::ToSql;

pub trait PrefixedId {
    const PREFIX: &str;
}

#[derive(FromSqlRow, AsExpression)]
#[diesel(sql_type = diesel::sql_types::Uuid)]
pub struct Id<T: PrefixedId> {
    id: ulid::Ulid,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: PrefixedId> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.id, f)
    }
}

impl<T: PrefixedId> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: PrefixedId> Eq for Id<T> {}

impl<T: PrefixedId> Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> Deref for Id<T>
where
    T: PrefixedId,
{
    type Target = ulid::Ulid;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl<T, I> From<I> for Id<T>
where
    T: PrefixedId,
    I: Into<ulid::Ulid>,
{
    fn from(id: I) -> Self {
        Self {
            id: id.into(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: PrefixedId> Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut id_str = self.id.to_string();
        id_str.make_ascii_lowercase();
        write!(f, "{}_{}", T::PREFIX, id_str)
    }
}

impl<T> FromSql<diesel::sql_types::Uuid, diesel::pg::Pg> for Id<T>
where
    T: PrefixedId,
{
    fn from_sql(bytes: diesel::pg::PgValue<'_>) -> diesel::deserialize::Result<Self> {
        let uuid = uuid::Uuid::from_sql(bytes)?;

        Ok(Self {
            id: ulid::Ulid::from(uuid),
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<T> ToSql<diesel::sql_types::Uuid, diesel::pg::Pg> for Id<T>
where
    T: PrefixedId + Debug,
{
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>) -> diesel::serialize::Result {
        out.write_all(&self.id.to_bytes())
            .map(|_| diesel::serialize::IsNull::No)
            .map_err(Into::into)
    }
}
