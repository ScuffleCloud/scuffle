use crate::id::{PolicySetId, RoleId};

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::role_policy_set_assignments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RolePolicySetAssignment {
    pub role_id: RoleId,
    pub policy_set_id: PolicySetId,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable)]
#[diesel(table_name = crate::schema::role_policy_set_assignments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(role_id, policy_set_id))]
pub struct NewRolePolicySetAssignment {
    pub role_id: RoleId,
    pub policy_set_id: PolicySetId,
}
