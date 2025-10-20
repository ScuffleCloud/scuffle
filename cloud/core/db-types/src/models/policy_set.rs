use std::borrow::Cow;

use crate::models::OrganizationId;

id::impl_id!(pub PolicySetId, "ps_");

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::policy_sets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PolicySet {
    pub id: PolicySetId,
    pub organization_id: Option<OrganizationId>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::policy_sets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPolicySet<'a> {
    #[builder(default)]
    pub id: PolicySetId,
    pub organization_id: Option<OrganizationId>,
    pub data: Cow<'a, [u8]>,
}
