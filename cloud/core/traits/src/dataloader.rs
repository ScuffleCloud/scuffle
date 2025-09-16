use std::collections::HashMap;

use core_db_types::models::{Organization, OrganizationId, User, UserId};
use scuffle_batching::DataLoaderFetcher;

pub trait DataloaderInterface {
    fn user_loader(
        &self,
    ) -> &scuffle_batching::DataLoader<impl DataLoaderFetcher<Key = UserId, Value = User> + Send + Sync + 'static>;
    fn organization_loader(
        &self,
    ) -> &scuffle_batching::DataLoader<
        impl DataLoaderFetcher<Key = OrganizationId, Value = Organization> + Send + Sync + 'static,
    >;
}

pub trait DataLoader {
    type Key;
    type Value;
    type Error;

    fn load(&self, key: Self::Key) -> impl Future<Output = Result<Option<Self::Value>, Self::Error>>;
    fn load_many(
        &self,
        keys: impl IntoIterator<Item = Self::Key> + Send,
    ) -> impl Future<Output = Result<HashMap<Self::Key, Self::Value>, Self::Error>>;
}

impl<E> DataLoader for scuffle_batching::DataLoader<E>
where
    E: scuffle_batching::DataLoaderFetcher + Send + Sync + 'static,
{
    type Error = ();
    type Key = E::Key;
    type Value = E::Value;

    async fn load(&self, key: Self::Key) -> Result<Option<Self::Value>, Self::Error> {
        scuffle_batching::DataLoader::load(self, key).await
    }

    async fn load_many(
        &self,
        keys: impl IntoIterator<Item = Self::Key> + Send,
    ) -> Result<HashMap<Self::Key, Self::Value>, Self::Error> {
        scuffle_batching::DataLoader::load_many(self, keys).await
    }
}
