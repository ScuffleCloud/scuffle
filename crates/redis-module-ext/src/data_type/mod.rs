use std::collections::HashMap;
use std::ffi::CString;
use std::sync::{LazyLock, RwLock};

use redis_module::native_types::RedisType;
use redis_module::raw;

mod io;
pub use io::*;
mod methods;
pub use methods::*;

struct RedisModuleType {
    ptr: *mut raw::RedisModuleType,
}

unsafe impl Send for RedisModuleType {}
unsafe impl Sync for RedisModuleType {}

pub trait RedisDataType: Sized {
    #[doc(hidden)]
    const NAME: &'static str;
    #[doc(hidden)]
    const VERSION: i32;

    #[doc(hidden)]
    fn module_methods(ctx: &redis_module::Context) -> raw::RedisModuleTypeMethods;

    #[doc(hidden)]
    fn register(ctx: &redis_module::Context) -> Result<(), &'static str> {
        if Self::NAME.len() != 9 {
            let msg = "Redis requires the length of native type names to be exactly 9 characters";
            ctx.log_warning(&format!("{msg}, name is: '{}'", Self::NAME));
            return Err(msg);
        }

        let name = CString::new(Self::NAME).unwrap();

        let redis_type = unsafe {
            raw::RedisModule_CreateDataType.unwrap()(
                ctx.ctx,
                name.as_ptr(),
                Self::VERSION, // Encoding version
                &mut Self::module_methods(ctx),
            )
        };

        if Self::redis_ty(redis_type).is_none() {
            ctx.log_warning("Error: created data type is null");
            return Err("Error: created data type is null");
        }

        ctx.log_debug(&format!("Created new data type '{}'", Self::NAME).as_str());

        Ok(())
    }

    #[doc(hidden)]
    fn redis_ty(ptr: *mut raw::RedisModuleType) -> Option<redis_module::native_types::RedisType> {
        static TYPE_REGISTRY: LazyLock<RwLock<HashMap<&'static str, RedisModuleType>>> =
            LazyLock::new(|| Default::default());

        if !ptr.is_null() {
            TYPE_REGISTRY.write().unwrap().insert(Self::NAME, RedisModuleType { ptr });
        }

        let ptr = TYPE_REGISTRY.read().unwrap().get(Self::NAME)?.ptr;

        let ty = RedisType::new(Self::NAME, Self::VERSION, unsafe { std::mem::zeroed() });
        ty.raw_type.replace(ptr);
        Some(ty)
    }
}
