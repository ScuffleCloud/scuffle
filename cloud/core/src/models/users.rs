use cedar_policy::RestrictedExpression;
use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use crate::cedar::CedarEntity;
use crate::id::{Id, PrefixedId};

pub(crate) type UserId = Id<User>;

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: UserId,
    pub preferred_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password_hash: Option<String>,
    pub primary_email: Option<String>,
}

impl PrefixedId for User {
    const PREFIX: &'static str = "u";
}

impl CedarEntity for User {
    const ENTITY_TYPE: &'static str = "User";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }

    fn attributes(&self) -> std::collections::HashMap<String, cedar_policy::RestrictedExpression> {
        self.id.attributes()
    }
}

impl From<User> for pb::scufflecloud::core::v1::User {
    fn from(value: User) -> Self {
        pb::scufflecloud::core::v1::User {
            id: value.id.to_string(),
            preferred_name: value.preferred_name,
            first_name: value.first_name,
            last_name: value.last_name,
            primary_email: value.primary_email,
        }
    }
}

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug)]
#[diesel(table_name = crate::schema::user_emails)]
#[diesel(primary_key(email))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserEmail {
    pub email: String,
    pub user_id: UserId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl CedarEntity for UserEmail {
    const ENTITY_TYPE: &'static str = "UserEmail";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(&self.email)
    }

    fn attributes(&self) -> std::collections::HashMap<String, cedar_policy::RestrictedExpression> {
        [("email".to_string(), RestrictedExpression::new_string(self.email.clone()))]
            .into_iter()
            .collect()
    }
}

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug)]
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

impl CedarEntity for UserGoogleAccount {
    const ENTITY_TYPE: &'static str = "UserGoogleAccount";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(&self.sub)
    }

    fn attributes(&self) -> std::collections::HashMap<String, RestrictedExpression> {
        [("sub".to_string(), RestrictedExpression::new_string(self.sub.clone()))]
            .into_iter()
            .collect()
    }
}
