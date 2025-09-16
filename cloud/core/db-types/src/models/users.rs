use std::time::SystemTime;

use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use crate::id::{Id, PrefixedId};

pub type UserId = Id<User>;

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, AsChangeset, serde_derive::Serialize, Clone)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: UserId,
    pub preferred_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password_hash: Option<String>,
    pub primary_email: Option<String>,
    pub avatar_url: Option<String>,
}

impl PrefixedId for User {
    const PREFIX: &'static str = "u";
}

impl From<User> for pb::scufflecloud::core::v1::User {
    fn from(value: User) -> Self {
        pb::scufflecloud::core::v1::User {
            id: value.id.to_string(),
            preferred_name: value.preferred_name,
            first_name: value.first_name,
            last_name: value.last_name,
            primary_email: value.primary_email,
            avatar_url: value.avatar_url,
            created_at: Some(value.id.datetime().into()),
        }
    }
}

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::user_emails)]
#[diesel(primary_key(email))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserEmail {
    pub email: String,
    pub user_id: UserId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<UserEmail> for pb::scufflecloud::core::v1::UserEmail {
    fn from(value: UserEmail) -> Self {
        pb::scufflecloud::core::v1::UserEmail {
            email: value.email,
            created_at: Some(SystemTime::from(value.created_at).into()),
        }
    }
}

pub type NewUserEmailRequestId = Id<NewUserEmailRequest>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::new_user_email_requests)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUserEmailRequest {
    pub id: NewUserEmailRequestId,
    pub user_id: UserId,
    pub email: String,
    pub code: Vec<u8>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl PrefixedId for NewUserEmailRequest {
    const PREFIX: &'static str = "er";
}

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(primary_key(sub))]
#[diesel(table_name = crate::schema::user_google_accounts)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserGoogleAccount {
    pub sub: String,
    pub user_id: UserId,
    pub access_token: String,
    pub access_token_expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
