use std::borrow::Cow;

use chrono::{DateTime, Utc};

use crate::models::{OrganizationId, UserId};

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::organization_members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrganizationMember {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub invited_by_id: Option<UserId>,
    pub inline_policy: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::organization_members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(organization_id, user_id))]
pub struct NewOrganizationMember<'a> {
    #[builder(start_fn)]
    pub organization_id: OrganizationId,
    #[builder(start_fn)]
    pub user_id: UserId,
    pub invited_by_id: Option<UserId>,
    pub inline_policy: Option<Cow<'a, [u8]>>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: DateTime<Utc>,
}
