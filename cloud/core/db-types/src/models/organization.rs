use std::borrow::Cow;

use crate::id::{OrganizationId, UserId};

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::organizations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Organization {
    pub id: OrganizationId,
    pub name: String,
    pub owner_id: UserId,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::organizations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewOrganization<'a> {
    #[builder(default)]
    pub id: OrganizationId,
    pub name: Cow<'a, str>,
    pub owner_id: UserId,
}
