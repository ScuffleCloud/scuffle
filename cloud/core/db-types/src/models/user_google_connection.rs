use std::borrow::Cow;

use chrono::{DateTime, Utc};

use crate::models::UserId;

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::user_google_connections)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserGoogleConnection {
    pub sub: String,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::user_google_connections)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(sub))]
pub struct NewUserGoogleConnection<'a> {
    pub sub: Cow<'a, str>,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
}
