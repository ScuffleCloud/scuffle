use std::time::SystemTime;

use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use crate::cedar::CedarEntity;
use crate::id::{Id, PrefixedId};
use crate::models::users::{User, UserId};

pub type OrganizationId = Id<Organization>;

#[derive(
    Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize, Clone,
)]
#[diesel(table_name = crate::schema::organizations)]
#[diesel(belongs_to(User, foreign_key = owner_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Organization {
    pub id: OrganizationId,
    pub google_customer_id: Option<String>,
    pub google_hosted_domain: Option<String>,
    pub name: String,
    pub owner_id: UserId,
}

impl PrefixedId for Organization {
    const PREFIX: &'static str = "o";
}

impl CedarEntity for Organization {
    const ENTITY_TYPE: &'static str = "Organization";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }
}

impl From<Organization> for pb::scufflecloud::core::v1::Organization {
    fn from(value: Organization) -> Self {
        pb::scufflecloud::core::v1::Organization {
            id: value.id.to_string(),
            google_hosted_domain: value.google_hosted_domain,
            name: value.name,
            owner_id: value.owner_id.to_string(),
            created_at: Some(value.id.datetime().into()),
        }
    }
}

pub type ProjectId = Id<Project>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::projects)]
#[diesel(belongs_to(Organization))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub organization_id: OrganizationId,
}

impl PrefixedId for Project {
    const PREFIX: &'static str = "p";
}

impl CedarEntity for Project {
    const ENTITY_TYPE: &'static str = "Project";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }
}

impl From<Project> for pb::scufflecloud::core::v1::Project {
    fn from(value: Project) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            organization_id: value.organization_id.to_string(),
            created_at: Some(value.id.datetime().into()),
        }
    }
}

pub type PolicyId = Id<Policy>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::policies)]
#[diesel(belongs_to(Organization))]
#[diesel(belongs_to(Project))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Policy {
    pub id: PolicyId,
    pub organization_id: OrganizationId,
    pub project_id: Option<ProjectId>,
    pub name: String,
    pub description: Option<String>,
    pub policy: String,
}

impl PrefixedId for Policy {
    const PREFIX: &'static str = "po";
}

impl CedarEntity for Policy {
    const ENTITY_TYPE: &'static str = "Policy";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }
}

pub type RoleId = Id<Role>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::roles)]
#[diesel(belongs_to(Organization))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Role {
    pub id: RoleId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub inline_policy: Option<String>,
}

impl PrefixedId for Role {
    const PREFIX: &'static str = "r";
}

impl CedarEntity for Role {
    const ENTITY_TYPE: &'static str = "Role";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }
}

#[derive(Queryable, Selectable, Insertable, Identifiable, Associations, Debug)]
#[diesel(table_name = crate::schema::role_policies)]
#[diesel(primary_key(role_id, policy_id))]
#[diesel(belongs_to(Role))]
#[diesel(belongs_to(Policy))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RolePolicy {
    pub role_id: RoleId,
    pub policy_id: PolicyId,
}

#[derive(
    Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize, Clone,
)]
#[diesel(table_name = crate::schema::organization_members)]
#[diesel(primary_key(organization_id, user_id))]
#[diesel(belongs_to(Organization))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrganizationMember {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub invited_by_id: Option<UserId>,
    pub inline_policy: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl CedarEntity for OrganizationMember {
    const ENTITY_TYPE: &'static str = "OrganizationMember";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(format!(
            "{}:{}",
            self.organization_id.to_string_unprefixed(),
            self.user_id.to_string_unprefixed()
        ))
    }

    // async fn parents(&self, _global: &Arc<G>) -> Result<HashSet<cedar_policy::EntityUid>, tonic::Status> {
    //     Ok(std::iter::once(CedarEntity::<G>::entity_uid(&self.organization_id)).collect())
    // }
}

impl From<OrganizationMember> for pb::scufflecloud::core::v1::OrganizationMember {
    fn from(value: OrganizationMember) -> Self {
        Self {
            organization_id: value.organization_id.to_string(),
            user_id: value.user_id.to_string(),
            invited_by_id: value.invited_by_id.map(|id| id.to_string()),
            created_at: Some(SystemTime::from(value.created_at).into()),
        }
    }
}

#[derive(Queryable, Selectable, Insertable, Identifiable, Associations, Debug)]
#[diesel(table_name = crate::schema::organization_member_policies)]
#[diesel(primary_key(organization_id, user_id, policy_id))]
#[diesel(belongs_to(Organization))]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Policy))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrganizationMemberPolicy {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub policy_id: PolicyId,
}

pub type ServiceAccountId = Id<ServiceAccount>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::service_accounts)]
#[diesel(belongs_to(Organization))]
#[diesel(belongs_to(Project))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServiceAccount {
    pub id: ServiceAccountId,
    pub name: String,
    pub organization_id: OrganizationId,
    pub project_id: Option<ProjectId>,
    pub inline_policy: Option<String>,
}

impl PrefixedId for ServiceAccount {
    const PREFIX: &'static str = "sa";
}

impl CedarEntity for ServiceAccount {
    const ENTITY_TYPE: &'static str = "ServiceAccount";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }

    // async fn parents(&self, _global: &Arc<G>) -> Result<HashSet<cedar_policy::EntityUid>, tonic::Status> {
    //     let mut parents = HashSet::new();
    //     parents.insert(CedarEntity::<G>::entity_uid(&self.organization_id));
    //     if let Some(project_id) = &self.project_id {
    //         parents.insert(CedarEntity::<G>::entity_uid(project_id));
    //     }
    //     Ok(parents)
    // }
}

pub type ServiceAccountTokenId = Id<ServiceAccountToken>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::service_account_tokens)]
#[diesel(belongs_to(ServiceAccount))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServiceAccountToken {
    pub id: ServiceAccountTokenId,
    pub active: bool,
    pub service_account_id: ServiceAccountId,
    pub token: Vec<u8>,
    pub inline_policy: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl PrefixedId for ServiceAccountToken {
    const PREFIX: &'static str = "sat";
}

impl CedarEntity for ServiceAccountToken {
    const ENTITY_TYPE: &'static str = "ServiceAccountToken";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }
}

pub type OrganizationInvitationId = Id<OrganizationInvitation>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::organization_invitations)]
#[diesel(belongs_to(Organization))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrganizationInvitation {
    pub id: OrganizationInvitationId,
    pub user_id: Option<UserId>,
    pub organization_id: OrganizationId,
    pub email: String,
    pub invited_by_id: UserId,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl PrefixedId for OrganizationInvitation {
    const PREFIX: &'static str = "oi";
}

impl CedarEntity for OrganizationInvitation {
    const ENTITY_TYPE: &'static str = "OrganizationInvitation";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }

    // async fn additional_attributes(
    //     &self,
    //     global: &Arc<G>,
    // ) -> Result<serde_json::value::Map<String, serde_json::Value>, tonic::Status> {
    //     let organization = common::get_organization_by_id(global, self.organization_id).await?;
    //     let organization_attr = organization.attributes(global).await?;

    //     Ok([("organization".to_string(), serde_json::Value::Object(organization_attr))]
    //         .into_iter()
    //         .collect())
    // }
}

impl From<OrganizationInvitation> for pb::scufflecloud::core::v1::OrganizationInvitation {
    fn from(value: OrganizationInvitation) -> Self {
        Self {
            id: value.id.to_string(),
            user_id: value.user_id.map(|id| id.to_string()),
            organization_id: value.organization_id.to_string(),
            email: value.email,
            invited_by_id: value.invited_by_id.to_string(),
            expires_at: value.expires_at.map(|dt| SystemTime::from(dt).into()),
            created_at: Some(value.id.datetime().into()),
        }
    }
}
