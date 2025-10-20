use std::borrow::Cow;

use crate::models::OrganizationId;

id::impl_id!(pub RoleId, "rl_");

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Role {
    pub id: RoleId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub inline_policy_set: Vec<u8>,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewRole<'a> {
    #[builder(default)]
    pub id: RoleId,
    pub organization_id: OrganizationId,
    pub name: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    pub inline_policy_set: Cow<'a, [u8]>,
}
