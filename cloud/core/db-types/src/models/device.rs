use std::borrow::Cow;

use chrono::{DateTime, Utc};

use crate::models::device_algorithm::DeviceAlgorithm;
use crate::models::sha256::Sha256;

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(fingerprint))]
pub struct Device {
    pub fingerprint: Sha256,
    pub algorithm: DeviceAlgorithm,
    pub public_key_data: Vec<u8>,
    pub last_active_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(fingerprint))]
pub struct NewDevice<'a> {
    #[builder(start_fn)]
    pub algorithm: DeviceAlgorithm,
    #[builder(start_fn)]
    pub public_key_data: Cow<'a, [u8]>,
    #[builder(field = Sha256::new(public_key_data.as_ref()))]
    pub fingerprint: Sha256,
    #[builder(default = chrono::Utc::now())]
    pub last_active_at: DateTime<Utc>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: DateTime<Utc>,
}
