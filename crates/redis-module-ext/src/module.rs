use std::ffi::CStr;

pub trait RedisModule: Sized {
    fn name() -> &'static CStr;
    fn version() -> i32;

    fn register_data_types(ctx: &redis_module::Context) -> bool;
    fn register_commands(ctx: &redis_module::Context) -> redis_module::RedisResult<()>;

    fn init_fn(ctx: &redis_module::Context) -> redis_module::RedisResult<()> {
        let _ = ctx;
        Ok(())
    }

    fn deinit_fn(ctx: &redis_module::Context) -> redis_module::RedisResult<()> {
        let _ = ctx;
        Ok(())
    }
}
