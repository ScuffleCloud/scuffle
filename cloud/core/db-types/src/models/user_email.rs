use std::borrow::Cow;

use chrono::{DateTime, Utc};

use crate::id::UserId;

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::user_emails)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserEmail {
    pub email: String,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::user_emails)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(email))]
pub struct NewUserEmail<'a> {
    pub email: Cow<'a, str>,
    pub user_id: UserId,
    #[builder(default = chrono::Utc::now())]
    pub created_at: DateTime<Utc>,
}
