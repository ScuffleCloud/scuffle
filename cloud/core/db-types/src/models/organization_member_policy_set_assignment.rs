use crate::models::{OrganizationId, PolicySetId, UserId};

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::organization_member_policy_set_assignments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrganizationMemberPolicySetAssignment {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub policy_set_id: PolicySetId,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable)]
#[diesel(table_name = crate::schema::organization_member_policy_set_assignments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(organization_id, user_id, policy_set_id))]
pub struct NewOrganizationMemberPolicySetAssignment {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub policy_set_id: PolicySetId,
}
