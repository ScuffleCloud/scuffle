use db_types::models::{Organization, OrganizationId, User, UserId};
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
