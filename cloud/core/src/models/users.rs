use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use crate::id::{Id, PrefixedId};

pub type UserId = Id<User>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: UserId,
    pub preferred_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password_hash: Option<String>,
    pub primary_email: String,
}

impl PrefixedId for User {
    const PREFIX: &'static str = "user";
}

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations)]
#[diesel(table_name = crate::schema::user_emails)]
#[diesel(primary_key(email))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserEmail {
    pub email: String,
    pub user_id: UserId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations)]
#[diesel(table_name = crate::schema::user_google_accounts)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserGoogleAccount {
    pub id: String,
    pub user_id: UserId,
    pub access_token: String,
    pub refresh_token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
