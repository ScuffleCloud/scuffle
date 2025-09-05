use std::collections::{HashMap, HashSet};
use std::time::Duration;

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::pooled_connection::bb8::PooledConnection;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use scuffle_batching::{DataLoader, DataLoaderFetcher};
use tokio::sync::Mutex;

use crate::models::{Organization, OrganizationId};
use crate::schema::users;

pub struct OrganizationLoader(Mutex<PooledConnection<'static, AsyncPgConnection>>);

impl DataLoaderFetcher for OrganizationLoader {
    type Key = OrganizationId;
    type Value = Organization;

    async fn load(&self, keys: HashSet<Self::Key>) -> Option<HashMap<Self::Key, Self::Value>> {
        let users = users::dsl::users
            .filter(users::dsl::id.eq_any(keys))
            .select(Organization::as_select())
            .load::<Organization>(&mut self.0.lock().await)
            .await
            .map_err(|e| tracing::error!(err = %e, "failed to load organizations"))
            .ok()?;

        Some(users.into_iter().map(|o| (o.id, o)).collect())
    }
}

impl OrganizationLoader {
    pub fn new(conn: PooledConnection<'static, AsyncPgConnection>) -> DataLoader<Self> {
        DataLoader::new(Self(Mutex::new(conn)), 1000, 500, Duration::from_millis(5))
    }
}
