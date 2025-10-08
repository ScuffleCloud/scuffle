use core_db_types::models::ProjectId;
use diesel::Selectable;
use diesel::prelude::{AsChangeset, Identifiable, Insertable, Queryable};
use id::impl_id;

impl_id!(pub StreamId, "s_");

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::streams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Stream {
    pub id: StreamId,
    pub project_id: ProjectId,
    pub name: String,
}
