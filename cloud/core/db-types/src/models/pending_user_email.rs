use std::borrow::Cow;

use chrono::{DateTime, Utc};

use crate::id::{PendingUserEmailId, UserId};

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::pending_user_emails)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PendingUserEmail {
    pub id: PendingUserEmailId,
    pub email: String,
    pub user_id: UserId,
    pub last_sent_code_at: DateTime<Utc>,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::pending_user_emails)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPendingUserEmail<'a> {
    #[builder(default)]
    pub id: PendingUserEmailId,
    pub email: Cow<'a, str>,
    pub user_id: UserId,
    #[builder(default = chrono::Utc::now())]
    pub last_sent_code_at: DateTime<Utc>,
}
