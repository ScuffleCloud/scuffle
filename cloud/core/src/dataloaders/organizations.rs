use std::collections::{HashMap, HashSet};
use std::time::Duration;

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::pooled_connection::bb8::PooledConnection;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use itertools::Itertools;
use scuffle_batching::{DataLoader, DataLoaderFetcher};
use tokio::sync::Mutex;

use crate::models::{Organization, OrganizationId, OrganizationMember, UserId};
use crate::schema::{organization_members, organizations};

pub struct OrganizationLoader(Mutex<PooledConnection<'static, AsyncPgConnection>>);

impl DataLoaderFetcher for OrganizationLoader {
    type Key = OrganizationId;
    type Value = Organization;

    async fn load(&self, keys: HashSet<Self::Key>) -> Option<HashMap<Self::Key, Self::Value>> {
        let organizations = organizations::dsl::organizations
            .filter(organizations::dsl::id.eq_any(keys))
            .select(Organization::as_select())
            .load::<Organization>(&mut self.0.lock().await)
            .await
            .map_err(|e| tracing::error!(err = %e, "failed to load organizations"))
            .ok()?;

        Some(organizations.into_iter().map(|o| (o.id, o)).collect())
    }
}

impl OrganizationLoader {
    pub fn new(conn: PooledConnection<'static, AsyncPgConnection>) -> DataLoader<Self> {
        DataLoader::new(Self(Mutex::new(conn)), 1000, 500, Duration::from_millis(5))
    }
}

pub struct OrganizationMemberByUserIdLoader(Mutex<PooledConnection<'static, AsyncPgConnection>>);

impl DataLoaderFetcher for OrganizationMemberByUserIdLoader {
    type Key = UserId;
    type Value = Vec<OrganizationMember>;

    async fn load(&self, keys: HashSet<Self::Key>) -> Option<HashMap<Self::Key, Self::Value>> {
        let organization_members = organization_members::dsl::organization_members
            .filter(organization_members::dsl::user_id.eq_any(keys))
            .select(OrganizationMember::as_select())
            .load::<OrganizationMember>(&mut self.0.lock().await)
            .await
            .map_err(|e| tracing::error!(err = %e, "failed to load organization members"))
            .ok()?;

        Some(organization_members.into_iter().into_group_map_by(|m| m.user_id))
    }
}

impl OrganizationMemberByUserIdLoader {
    pub fn new(conn: PooledConnection<'static, AsyncPgConnection>) -> DataLoader<Self> {
        DataLoader::new(Self(Mutex::new(conn)), 1000, 500, Duration::from_millis(5))
    }
}
