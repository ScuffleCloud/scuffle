use std::collections::{HashMap, HashSet};
use std::time::Duration;

use core_db_types::models::{Organization, OrganizationId, OrganizationMember, UserId};
use core_db_types::schema::{organization_members, organizations};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::pooled_connection::bb8;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use itertools::Itertools;
use scuffle_batching::{DataLoader, DataLoaderFetcher};

pub(crate) struct OrganizationLoader(bb8::Pool<AsyncPgConnection>);

impl DataLoaderFetcher for OrganizationLoader {
    type Key = OrganizationId;
    type Value = Organization;

    async fn load(&self, keys: HashSet<Self::Key>) -> Option<HashMap<Self::Key, Self::Value>> {
        let mut conn = self
            .0
            .get()
            .await
            .map_err(|e| tracing::error!(err = %e, "failed to get connection"))
            .ok()?;

        let organizations = organizations::dsl::organizations
            .filter(organizations::dsl::id.eq_any(keys))
            .select(Organization::as_select())
            .load::<Organization>(&mut conn)
            .await
            .map_err(|e| tracing::error!(err = %e, "failed to load organizations"))
            .ok()?;

        Some(organizations.into_iter().map(|o| (o.id, o)).collect())
    }
}

impl OrganizationLoader {
    pub(crate) fn new(pool: bb8::Pool<AsyncPgConnection>) -> DataLoader<Self> {
        DataLoader::new(Self(pool), 1000, 500, Duration::from_millis(5))
    }
}

pub(crate) struct OrganizationMemberByUserIdLoader(bb8::Pool<AsyncPgConnection>);

impl DataLoaderFetcher for OrganizationMemberByUserIdLoader {
    type Key = UserId;
    type Value = Vec<OrganizationMember>;

    async fn load(&self, keys: HashSet<Self::Key>) -> Option<HashMap<Self::Key, Self::Value>> {
        let mut conn = self
            .0
            .get()
            .await
            .map_err(|e| tracing::error!(err = %e, "failed to get connection"))
            .ok()?;

        let organization_members = organization_members::dsl::organization_members
            .filter(organization_members::dsl::user_id.eq_any(keys))
            .select(OrganizationMember::as_select())
            .load::<OrganizationMember>(&mut conn)
            .await
            .map_err(|e| tracing::error!(err = %e, "failed to load organization members"))
            .ok()?;

        Some(organization_members.into_iter().into_group_map_by(|m| m.user_id))
    }
}

impl OrganizationMemberByUserIdLoader {
    pub(crate) fn new(pool: bb8::Pool<AsyncPgConnection>) -> DataLoader<Self> {
        DataLoader::new(Self(pool), 1000, 500, Duration::from_millis(5))
    }
}
