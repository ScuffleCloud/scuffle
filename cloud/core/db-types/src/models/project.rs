use std::borrow::Cow;

use crate::models::OrganizationId;

id::impl_id!(pub ProjectId, "prj_");

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::projects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::projects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewProject<'a> {
    #[builder(default)]
    pub id: ProjectId,
    pub name: Cow<'a, str>,
    pub organization_id: OrganizationId,
}
