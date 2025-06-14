use redis_module_ext::{export_redis_module, redis_module};

mod quota;
mod utils;

#[redis_module(name = "scuffle", version = 1, merge(quota::QuotaModule))]
struct ScuffleModule;

export_redis_module!(ScuffleModule);
