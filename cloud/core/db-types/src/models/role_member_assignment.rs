use crate::models::{OrganizationId, RoleId, UserId};

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::role_member_assignments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RoleMemberAssignment {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub role_id: RoleId,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable)]
#[diesel(table_name = crate::schema::role_member_assignments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(organization_id, user_id, role_id))]
pub struct NewRoleMemberAssignment {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub role_id: RoleId,
}
