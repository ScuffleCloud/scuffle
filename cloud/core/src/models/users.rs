#![allow(dead_code)]

use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    #[diesel(deserialize_as = uuid::Uuid)]
    pub id: ulid::Ulid,
}
