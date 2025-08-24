use std::collections::{HashMap, HashSet};
use std::time::Duration;

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::pooled_connection::bb8::PooledConnection;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use scuffle_batching::{DataLoader, DataLoaderFetcher};
use tokio::sync::Mutex;

use crate::models::{User, UserId};
use crate::schema::users;

pub struct UserLoader(Mutex<PooledConnection<'static, AsyncPgConnection>>);

impl DataLoaderFetcher for UserLoader {
    type Key = UserId;
    type Value = User;

    async fn load(&self, keys: HashSet<Self::Key>) -> Option<HashMap<Self::Key, Self::Value>> {
        let users = users::dsl::users
            .filter(users::dsl::id.eq_any(keys))
            .select(User::as_select())
            .load::<User>(&mut self.0.lock().await)
            .await
            .map_err(|e| tracing::error!(err = %e, "failed to load users"))
            .ok()?;

        Some(users.into_iter().map(|u| (u.id, u)).collect())
    }
}

impl<'a> UserLoader {
    pub fn new(conn: PooledConnection<'static, AsyncPgConnection>) -> DataLoader<Self> {
        DataLoader::new(Self(Mutex::new(conn)), 1000, 500, Duration::from_millis(5))
    }
}
