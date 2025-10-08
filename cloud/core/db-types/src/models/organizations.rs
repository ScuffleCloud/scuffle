use std::time::SystemTime;

use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use crate::models::users::{User, UserId};

id::impl_id!(pub OrganizationId, "o_");

#[derive(
    Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize, Clone,
)]
#[diesel(table_name = crate::schema::organizations)]
#[diesel(belongs_to(User, foreign_key = owner_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Organization {
    pub id: OrganizationId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_customer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_hosted_domain: Option<String>,
    pub name: String,
    pub owner_id: UserId,
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

id::impl_id!(pub ProjectId, "p_");

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::projects)]
#[diesel(belongs_to(Organization))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub organization_id: OrganizationId,
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

id::impl_id!(pub PolicyId, "po_");

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::policies)]
#[diesel(belongs_to(Organization))]
#[diesel(belongs_to(Project))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Policy {
    pub id: PolicyId,
    pub organization_id: OrganizationId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<ProjectId>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub policy: String,
}

id::impl_id!(pub RoleId, "r_");

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::roles)]
#[diesel(belongs_to(Organization))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Role {
    pub id: RoleId,
    pub organization_id: OrganizationId,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_policy: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invited_by_id: Option<UserId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_policy: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
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

id::impl_id!(pub ServiceAccountId, "sa_");

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::service_accounts)]
#[diesel(belongs_to(Organization))]
#[diesel(belongs_to(Project))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServiceAccount {
    pub id: ServiceAccountId,
    pub name: String,
    pub organization_id: OrganizationId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<ProjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_policy: Option<String>,
}

id::impl_id!(pub ServiceAccountTokenId, "sat_");

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::service_account_tokens)]
#[diesel(belongs_to(ServiceAccount))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServiceAccountToken {
    pub id: ServiceAccountTokenId,
    pub active: bool,
    pub service_account_id: ServiceAccountId,
    pub token: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

id::impl_id!(pub OrganizationInvitationId, "oi_");

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::organization_invitations)]
#[diesel(belongs_to(Organization))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrganizationInvitation {
    pub id: OrganizationInvitationId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,
    pub organization_id: OrganizationId,
    pub email: String,
    pub invited_by_id: UserId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
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
