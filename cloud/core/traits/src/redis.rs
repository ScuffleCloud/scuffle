pub trait RedisInterface: Send + Sync {
    type RedisConnection<'a>: fred::interfaces::ClientLike + fred::interfaces::KeysInterface
    where
        Self: 'a;

    fn redis(&self) -> &Self::RedisConnection<'_>;
}
