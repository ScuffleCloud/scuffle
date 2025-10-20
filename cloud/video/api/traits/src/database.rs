pub trait DatabaseInterface: Send + Sync {
    type Connection<'a>: diesel_async::AsyncConnection<Backend = diesel::pg::Pg>
    where
        Self: 'a;

    fn db(&self) -> impl std::future::Future<Output = anyhow::Result<Self::Connection<'_>>> + Send;
}
