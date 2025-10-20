use std::borrow::Cow;

use chrono::{DateTime, Utc};

use crate::models::UserId;

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::pending_user_emails)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(email, user_id))]
pub struct PendingUserEmail {
    pub email: String,
    pub user_id: UserId,
    pub codes_sent: i32,
    pub last_sent_code_at: DateTime<Utc>,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::pending_user_emails)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(email, user_id))]
pub struct NewPendingUserEmail<'a> {
    pub email: Cow<'a, str>,
    pub user_id: UserId,
    #[builder(default = 1)]
    pub codes_sent: i32,
    #[builder(default = chrono::Utc::now())]
    pub last_sent_code_at: DateTime<Utc>,
}
