use redis_module::raw;

pub fn redis_time_millis() -> i64 {
    unsafe { raw::RedisModule_Milliseconds.unwrap()() }
}

pub fn redis_time_micro() -> i64 {
    unsafe { raw::RedisModule_Microseconds.unwrap()() }
}

pub fn redis_time_cached_micro() -> i64 {
    unsafe { raw::RedisModule_CachedMicroseconds.unwrap()() }
}
