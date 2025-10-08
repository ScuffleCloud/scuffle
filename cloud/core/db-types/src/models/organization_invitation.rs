use std::borrow::Cow;

use chrono::{DateTime, Utc};

use crate::id::{OrganizationId, OrganizationInvitationId, UserId};

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::organization_invitations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrganizationInvitation {
    pub id: OrganizationInvitationId,
    pub organization_id: OrganizationId,
    pub email: String,
    pub invited_by_id: UserId,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::organization_invitations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(id))]
pub struct NewOrganizationInvitation<'a> {
    #[builder(default)]
    pub id: OrganizationInvitationId,
    pub organization_id: OrganizationId,
    pub email: Cow<'a, str>,
    pub invited_by_id: UserId,
    pub expires_at: Option<DateTime<Utc>>,
}
