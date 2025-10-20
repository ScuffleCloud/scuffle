#[derive(
    Debug, PartialEq, Eq, Clone, Copy, diesel_derive_enum::DbEnum, serde_derive::Deserialize, serde_derive::Serialize,
)]
#[db_enum(existing_type_path = "crate::schema::sql_types::DeviceAlgorithm")]
pub enum DeviceAlgorithm {
    #[db_enum(rename = "RSA_OAEP_SHA256")]
    #[serde(rename = "RSA_OAEP_SHA256")]
    RsaOaepSha256,
}
