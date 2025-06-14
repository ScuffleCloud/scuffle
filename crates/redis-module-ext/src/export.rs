use std::ffi::c_int;

use redis_module::raw;

use crate::RedisModule;

#[macro_export]
macro_rules! export_redis_module {
    ($module:ty) => {
        const _: () = {
            /// Redis module allocator.
            #[global_allocator]
            static REDIS_MODULE_ALLOCATOR: $crate::redis::alloc::RedisAlloc = $crate::redis::alloc::RedisAlloc;

            #[unsafe(no_mangle)]
            #[allow(non_snake_case, unsafe_op_in_unsafe_fn)]
            pub unsafe extern "C" fn RedisModule_OnLoad(
                ctx: *mut $crate::raw::RedisModuleCtx,
                argv: *mut *mut $crate::raw::RedisModuleString,
                argc: std::os::raw::c_int,
            ) -> std::os::raw::c_int {
                unsafe { $crate::export::on_load::<$module>(ctx, argv, argc) }
            }

            #[unsafe(no_mangle)]
            #[allow(non_snake_case, unsafe_op_in_unsafe_fn)]
            pub extern "C" fn RedisModule_OnUnload(ctx: *mut $crate::raw::RedisModuleCtx) -> std::os::raw::c_int {
                unsafe { $crate::export::on_unload::<$module>(ctx) }
            }
        };
    };
}

pub unsafe fn on_load<M: RedisModule>(
    ctx: *mut raw::RedisModuleCtx,
    _: *mut *mut raw::RedisModuleString,
    _: std::os::raw::c_int,
) -> std::os::raw::c_int {
    let name = M::name();
    let version = M::version();

    let result = unsafe { raw::Export_RedisModule_Init(ctx, name.as_ptr(), version, raw::REDISMODULE_APIVER_1 as c_int) };

    if result == raw::Status::Err as c_int {
        return raw::Status::Err as c_int;
    }

    let context = redis_module::Context::new(ctx);
    let _ = redis_module::MODULE_CONTEXT.set_context(&context);

    if !M::register_data_types(&context) {
        return raw::Status::Err as c_int;
    }

    if let Err(e) = M::register_commands(&context) {
        context.log_warning(&e.to_string());
    }

    if let Err(e) = redis_module::defrag::register_defrag_functions(&context) {
        context.log_warning(&e.to_string());
        return raw::Status::Err as c_int;
    }

    extern "C" fn __info_func(ctx: *mut raw::RedisModuleInfoCtx, for_crash_report: i32) {
        redis_module::basic_info_command_handler(&redis_module::InfoContext::new(ctx), for_crash_report == 1);
    }

    raw::register_info_function(ctx, Some(__info_func));

    if let Err(e) = redis_module::server_events::register_server_events(&context) {
        context.log_warning(&e.to_string());
        return raw::Status::Err as c_int;
    }

    if let Err(err) = M::init_fn(&context) {
        context.log_warning(&err.to_string());
        return raw::Status::Err as c_int;
    }

    raw::Status::Ok as c_int
}

pub unsafe fn on_unload<M: RedisModule>(ctx: *mut raw::RedisModuleCtx) -> std::os::raw::c_int {
    let context = redis_module::Context::new(ctx);

    if let Err(err) = M::deinit_fn(&context) {
        context.log_warning(&err.to_string());
        return raw::Status::Err as c_int;
    }

    raw::Status::Ok as c_int
}
