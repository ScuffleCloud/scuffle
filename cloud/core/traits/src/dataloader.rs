use core_db_types::models::{Organization, OrganizationId, OrganizationMember, User, UserId};
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
    fn organization_member_by_user_id_loader(
        &self,
    ) -> &scuffle_batching::DataLoader<
        impl DataLoaderFetcher<Key = UserId, Value = Vec<OrganizationMember>> + Send + Sync + 'static,
    >;
}
