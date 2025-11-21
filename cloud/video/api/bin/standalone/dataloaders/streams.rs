use std::collections::{HashMap, HashSet};
use std::time::Duration;

use db_types::models::{Stream, StreamId};
use db_types::schema::streams;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::pooled_connection::bb8;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use scuffle_batching::{DataLoader, DataLoaderFetcher};

pub(crate) struct StreamLoader(bb8::Pool<AsyncPgConnection>);

impl DataLoaderFetcher for StreamLoader {
    type Key = StreamId;
    type Value = Stream;

    async fn load(&self, keys: HashSet<Self::Key>) -> Option<HashMap<Self::Key, Self::Value>> {
        let mut conn = self
            .0
            .get()
            .await
            .map_err(|e| tracing::error!(err = %e, "failed to get connection"))
            .ok()?;

        let streams = streams::dsl::streams
            .filter(streams::dsl::id.eq_any(keys))
            .select(Stream::as_select())
            .load::<Stream>(&mut conn)
            .await
            .map_err(|e| tracing::error!(err = %e, "failed to load streams"))
            .ok()?;

        Some(streams.into_iter().map(|u| (u.id, u)).collect())
    }
}

impl StreamLoader {
    pub(crate) fn new(pool: bb8::Pool<AsyncPgConnection>) -> DataLoader<Self> {
        DataLoader::new(Self(pool), 1000, 500, Duration::from_millis(5))
    }
}
