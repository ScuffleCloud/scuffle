mod quota;
mod utils;

#[cfg(not(test))]
use quota::QUOTA_TYPE;

#[cfg(not(test))]
redis_module::redis_module! {
    name: "scuffle",
    version: 1,
    allocator: (redis_module::alloc::RedisAlloc, redis_module::alloc::RedisAlloc),
    data_types: [
        QUOTA_TYPE,
    ],
    commands: [],
}
