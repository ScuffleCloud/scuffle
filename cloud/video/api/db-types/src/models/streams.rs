use core_db_types::models::ProjectId;
use diesel::Selectable;
use diesel::prelude::{AsChangeset, Identifiable, Insertable, Queryable};
use id::impl_id;

impl_id!(pub StreamId, "s_");

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Debug, serde_derive::Serialize, Clone)]
#[diesel(table_name = crate::schema::streams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Stream {
    pub id: StreamId,
    pub project_id: ProjectId,
    pub name: String,
}

impl From<Stream> for pb::scufflecloud::video::api::v1::Stream {
    fn from(value: Stream) -> Self {
        Self {
            id: value.id.to_string(),
            project_id: value.project_id.to_string(),
            name: value.name,
        }
    }
}
