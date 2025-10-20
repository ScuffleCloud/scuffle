use ext_traits::ResultExt;

pub trait DatabaseInterface: Send + Sync {
    type Connection<'a>: diesel_async::AsyncConnection<Backend = diesel::pg::Pg>
    where
        Self: 'a;

    fn db(&self) -> impl std::future::Future<Output = anyhow::Result<Self::Connection<'_>>> + Send;
}

pub trait DatabaseExt {
    fn tx<R>(&mut self, f: impl AsyncFnOnce(&mut Self) -> tonic::Result<R>) -> impl Future<Output = tonic::Result<R>>;
}

impl<T> DatabaseExt for T
where
    T: diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
{
    async fn tx<R>(&mut self, f: impl AsyncFnOnce(&mut Self) -> tonic::Result<R>) -> tonic::Result<R> {
        <T::TransactionManager as diesel_async::TransactionManager<Self>>::begin_transaction(self).await
            .into_tonic_internal_err("failed to begin transaction")?;

        match f(self).await {
            Ok(result) => {
                <T::TransactionManager as diesel_async::TransactionManager<Self>>::commit_transaction(self).await
                    .into_tonic_internal_err("failed to commit transaction")?;
                Ok(result)
            },
            Err(e) => {
                <T::TransactionManager as diesel_async::TransactionManager<Self>>::rollback_transaction(self).await
                    .into_tonic_internal_err("failed to rollback transaction")?;
                Err(e)
            }
        }
    }
}
