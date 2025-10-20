use std::borrow::Cow;

use chrono::{DateTime, Utc};

use crate::models::UserId;

id::impl_id!(pub MagicLinkRequestId, "mlr_");

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::magic_link_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MagicLinkRequest {
    pub id: MagicLinkRequestId,
    pub user_id: UserId,
    pub email: String,
    pub code: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::magic_link_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMagicLinkRequest<'a> {
    #[builder(default)]
    pub id: MagicLinkRequestId,
    pub user_id: UserId,
    pub email: Cow<'a, str>,
    pub code: Cow<'a, str>,
    pub expires_at: DateTime<Utc>,
}
