use redis_module::RedisResult;
use redis_module::logging::RedisLogLevel;

use crate::RedisDataType;

pub trait RedisKeyWritableExt {
    /// Get a custom [RedisDataType] from a key.
    fn get<T: RedisDataType>(&self) -> RedisResult<Option<&mut T>>;
    /// Set a key to a custom [RedisDataType].
    fn set<T: RedisDataType>(&self, value: T) -> RedisResult<()>;
}

impl RedisKeyWritableExt for redis_module::key::RedisKeyWritable {
    fn get<T: RedisDataType>(&self) -> RedisResult<Option<&mut T>> {
        let Some(ty) = T::redis_ty(std::ptr::null_mut()) else {
            return Err(redis_module::RedisError::Str("type has not been registered"));
        };

        self.get_value(&ty)
    }

    fn set<T: RedisDataType>(&self, value: T) -> RedisResult<()> {
        let Some(ty) = T::redis_ty(std::ptr::null_mut()) else {
            return Err(redis_module::RedisError::Str("type has not been registered"));
        };

        self.set_value(&ty, value)
    }
}

pub trait IoLoggingExt {
    fn log(&mut self, level: redis_module::logging::RedisLogLevel, message: impl AsRef<str>);

    fn log_warning(&mut self, message: impl AsRef<str>) {
        self.log(RedisLogLevel::Warning, message);
    }

    fn log_debug(&mut self, message: impl AsRef<str>) {
        self.log(RedisLogLevel::Debug, message);
    }

    fn log_notice(&mut self, message: impl AsRef<str>) {
        self.log(RedisLogLevel::Notice, message);
    }

    fn log_verbose(&mut self, message: impl AsRef<str>) {
        self.log(RedisLogLevel::Verbose, message);
    }
}
