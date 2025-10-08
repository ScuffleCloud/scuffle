use sha2::Digest;

#[derive(
    diesel::deserialize::FromSqlRow, diesel::expression::AsExpression, Debug, PartialEq, Eq, Hash, Clone, Copy, Default,
)]
#[diesel(sql_type = diesel::sql_types::Bytea)]
pub struct Sha256(pub [u8; 32]);

impl Sha256 {
    pub fn new(data: impl AsRef<[u8]>) -> Self {
        let hash = sha2::Sha256::digest(data.as_ref());
        Self(hash.as_slice().try_into().unwrap())
    }

    pub fn from_hash(hash: [u8; 32]) -> Self {
        Self(hash)
    }
}

impl diesel::deserialize::FromSql<diesel::sql_types::Bytea, diesel::pg::Pg> for Sha256 {
    fn from_sql(bytes: diesel::pg::PgValue<'_>) -> diesel::deserialize::Result<Self> {
        let bytes: *const [u8] = diesel::deserialize::FromSql::from_sql(bytes)?;
        // Safety: this is fine as the pointer is never null and the bytes are valid
        #[allow(unsafe_code)]
        let bytes = unsafe { &*bytes };
        Ok(Self(bytes.try_into()?))
    }
}

impl diesel::serialize::ToSql<diesel::sql_types::Bytea, diesel::pg::Pg> for Sha256 {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>) -> diesel::serialize::Result {
        let data: &[u8] = &self.0;
        diesel::serialize::ToSql::<diesel::sql_types::Bytea, diesel::pg::Pg>::to_sql(data, out)
    }
}
