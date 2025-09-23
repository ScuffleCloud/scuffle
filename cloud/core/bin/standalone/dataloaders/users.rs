use std::collections::{HashMap, HashSet};
use std::time::Duration;

use core_db_types::models::{User, UserId};
use core_db_types::schema::users;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::pooled_connection::bb8;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use scuffle_batching::{DataLoader, DataLoaderFetcher};

pub(crate) struct UserLoader(bb8::Pool<AsyncPgConnection>);

impl DataLoaderFetcher for UserLoader {
    type Key = UserId;
    type Value = User;

    async fn load(&self, keys: HashSet<Self::Key>) -> Option<HashMap<Self::Key, Self::Value>> {
        let mut conn = self
            .0
            .get()
            .await
            .map_err(|e| tracing::error!(err = %e, "failed to get connection"))
            .ok()?;

        let users = users::dsl::users
            .filter(users::dsl::id.eq_any(keys))
            .select(User::as_select())
            .load::<User>(&mut conn)
            .await
            .map_err(|e| tracing::error!(err = %e, "failed to load users"))
            .ok()?;

        Some(users.into_iter().map(|u| (u.id, u)).collect())
    }
}

impl UserLoader {
    pub(crate) fn new(pool: bb8::Pool<AsyncPgConnection>) -> DataLoader<Self> {
        DataLoader::new(Self(pool), 1000, 500, Duration::from_millis(5))
    }
}
