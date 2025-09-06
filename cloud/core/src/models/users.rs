use std::collections::HashSet;
use std::sync::Arc;

use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use crate::CoreConfig;
use crate::cedar::CedarEntity;
use crate::chrono_ext::ChronoDateTimeExt;
use crate::id::{Id, PrefixedId};
use crate::std_ext::OptionExt;

pub(crate) type UserId = Id<User>;

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, AsChangeset, serde::Serialize, Clone)]
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

impl<G: CoreConfig> CedarEntity<G> for User {
    const ENTITY_TYPE: &'static str = "User";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }

    async fn parents(&self, global: &Arc<G>) -> Result<HashSet<cedar_policy::EntityUid>, tonic::Status> {
        let organization_ids = global
            .organization_member_by_user_id_loader()
            .load(self.id)
            .await
            .ok()
            .into_tonic_internal_err("failed to query organization members")?
            .into_tonic_not_found("user not found")?
            .into_iter()
            .map(|m| m.organization_id)
            .map(|id| CedarEntity::<G>::entity_uid(&id))
            .collect::<HashSet<_>>();

        Ok(organization_ids)
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
            created_at: Some(tinc::well_known::prost::Timestamp::from(value.id.datetime())),
        }
    }
}

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde::Serialize)]
#[diesel(table_name = crate::schema::user_emails)]
#[diesel(primary_key(email))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserEmail {
    pub email: String,
    pub user_id: UserId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl<G> CedarEntity<G> for UserEmail {
    const ENTITY_TYPE: &'static str = "UserEmail";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(&self.email)
    }
}

impl From<UserEmail> for pb::scufflecloud::core::v1::UserEmail {
    fn from(value: UserEmail) -> Self {
        pb::scufflecloud::core::v1::UserEmail {
            email: value.email,
            created_at: Some(value.created_at.to_prost_timestamp_utc()),
        }
    }
}

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde::Serialize)]
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

impl<G> CedarEntity<G> for UserGoogleAccount {
    const ENTITY_TYPE: &'static str = "UserGoogleAccount";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(&self.sub)
    }
}
