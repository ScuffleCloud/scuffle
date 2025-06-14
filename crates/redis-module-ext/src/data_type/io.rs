use std::borrow::Borrow;
use std::hash::Hasher;
use std::ptr::NonNull;
use std::str::Utf8Error;

use redis_module::{RedisBuffer, RedisError, RedisResult, RedisString, raw};

use crate::data_type::methods::CStrConv;
use crate::prelude::IoLoggingExt;

pub struct RdbSaveIo {
    ptr: NonNull<raw::RedisModuleIO>,
}

impl RdbSaveIo {
    pub unsafe fn new(ptr: NonNull<raw::RedisModuleIO>) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> NonNull<raw::RedisModuleIO> {
        self.ptr
    }

    pub fn save_double(&mut self, value: f64) {
        raw::save_double(self.ptr.as_ptr(), value);
    }

    pub fn save_float(&mut self, value: f32) {
        raw::save_float(self.ptr.as_ptr(), value);
    }

    pub fn save_redis_string(&mut self, string: impl Borrow<RedisString>) {
        raw::save_redis_string(self.ptr.as_ptr(), string.borrow());
    }

    pub fn save_signed(&mut self, value: i64) {
        raw::save_signed(self.ptr.as_ptr(), value);
    }

    pub fn save_unsigned(&mut self, value: u64) {
        raw::save_unsigned(self.ptr.as_ptr(), value);
    }

    pub fn save_slice(&mut self, value: impl AsRef<[u8]>) {
        raw::save_slice(self.ptr.as_ptr(), value.as_ref());
    }

    pub fn save_str(&mut self, value: impl AsRef<str>) {
        raw::save_string(self.ptr.as_ptr(), value.as_ref());
    }
}

impl IoLoggingExt for RdbSaveIo {
    fn log(&mut self, level: redis_module::logging::RedisLogLevel, message: impl AsRef<str>) {
        redis_module::logging::log_io_error(self.ptr.as_ptr(), level, message.as_ref());
    }
}

pub struct RdbLoadIo {
    ptr: NonNull<raw::RedisModuleIO>,
}

impl RdbLoadIo {
    pub unsafe fn new(ptr: NonNull<raw::RedisModuleIO>) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> NonNull<raw::RedisModuleIO> {
        self.ptr
    }

    pub fn load_double(&mut self) -> RedisResult<f64> {
        raw::load_double(self.ptr.as_ptr()).map_err(|err| redis_module::RedisError::String(err.to_string()))
    }

    pub fn load_float(&mut self) -> RedisResult<f32> {
        raw::load_float(self.ptr.as_ptr()).map_err(|err| redis_module::RedisError::String(err.to_string()))
    }

    pub fn load_string(&mut self) -> RedisResult<RedisString> {
        raw::load_string(self.ptr.as_ptr()).map_err(|err| redis_module::RedisError::String(err.to_string()))
    }

    pub fn load_string_buffer(&mut self) -> RedisResult<RedisBuffer> {
        raw::load_string_buffer(self.ptr.as_ptr()).map_err(|err| redis_module::RedisError::String(err.to_string()))
    }

    pub fn load_unsigned(&mut self) -> RedisResult<u64> {
        raw::load_unsigned(self.ptr.as_ptr()).map_err(|err| redis_module::RedisError::String(err.to_string()))
    }

    pub fn load_signed(&mut self) -> RedisResult<i64> {
        raw::load_signed(self.ptr.as_ptr()).map_err(|err| redis_module::RedisError::String(err.to_string()))
    }

    pub fn log(&mut self, level: redis_module::logging::RedisLogLevel, message: impl AsRef<str>) {
        redis_module::logging::log_io_error(self.ptr.as_ptr(), level, message.as_ref());
    }

    pub fn log_warning(&mut self, message: impl AsRef<str>) {
        self.log(redis_module::logging::RedisLogLevel::Warning, message)
    }

    pub fn log_debug(&mut self, message: impl AsRef<str>) {
        self.log(redis_module::logging::RedisLogLevel::Debug, message)
    }

    pub fn log_notice(&mut self, message: impl AsRef<str>) {
        self.log(redis_module::logging::RedisLogLevel::Notice, message)
    }

    pub fn log_verbose(&mut self, message: impl AsRef<str>) {
        self.log(redis_module::logging::RedisLogLevel::Verbose, message)
    }
}

impl IoLoggingExt for RdbLoadIo {
    fn log(&mut self, level: redis_module::logging::RedisLogLevel, message: impl AsRef<str>) {
        redis_module::logging::log_io_error(self.ptr.as_ptr(), level, message.as_ref());
    }
}

pub struct AofRewriteIo {
    ptr: NonNull<raw::RedisModuleIO>,
}

#[must_use = "Does nothing unless `EmitAofCommandBuilder::dispatch` is called"]
pub struct EmitAofCommandBuilder<'a, C> {
    io: &'a mut AofRewriteIo,
    cmd: C,
    args: Vec<RedisString>,
}

impl<C> EmitAofCommandBuilder<'_, C> {
    pub fn arg(mut self, arg: impl AsRef<[u8]>) -> Self {
        self.args
            .push(RedisString::create_from_slice(std::ptr::null_mut(), arg.as_ref()));
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl AsRef<[u8]>>) -> Self {
        self.args.extend(
            args.into_iter()
                .map(|arg| RedisString::create_from_slice(std::ptr::null_mut(), arg.as_ref())),
        );
        self
    }

    pub fn dispatch(mut self) -> RedisResult<()>
    where
        C: CStrConv,
    {
        let command = unsafe { raw::RedisModule_EmitAOF }.ok_or(RedisError::Str("missing emit aof"))?;

        let cmd = self.cmd.into_cstr().ok_or(RedisError::Str("invalid command name"))?;
        let cmd = cmd.borrow();

        unsafe {
            command(
                self.io.ptr.as_ptr(),
                cmd.as_ptr(),
                c"v".as_ptr(),
                self.args.as_mut_ptr(),
                self.args.len(),
            )
        };
        Ok(())
    }
}

impl AofRewriteIo {
    pub unsafe fn new(ptr: NonNull<raw::RedisModuleIO>) -> Self {
        Self { ptr }
    }

    pub fn emit_command<C>(&mut self, cmd: C) -> EmitAofCommandBuilder<C> {
        EmitAofCommandBuilder {
            io: self,
            cmd,
            args: Vec::new(),
        }
    }

    pub fn log(&mut self, level: redis_module::logging::RedisLogLevel, message: impl AsRef<str>) {
        redis_module::logging::log_io_error(self.ptr.as_ptr(), level, message.as_ref());
    }

    pub fn log_warning(&mut self, message: impl AsRef<str>) {
        self.log(redis_module::logging::RedisLogLevel::Warning, message)
    }

    pub fn log_debug(&mut self, message: impl AsRef<str>) {
        self.log(redis_module::logging::RedisLogLevel::Debug, message)
    }

    pub fn log_notice(&mut self, message: impl AsRef<str>) {
        self.log(redis_module::logging::RedisLogLevel::Notice, message)
    }

    pub fn log_verbose(&mut self, message: impl AsRef<str>) {
        self.log(redis_module::logging::RedisLogLevel::Verbose, message)
    }
}

impl IoLoggingExt for AofRewriteIo {
    fn log(&mut self, level: redis_module::logging::RedisLogLevel, message: impl AsRef<str>) {
        redis_module::logging::log_io_error(self.ptr.as_ptr(), level, message.as_ref());
    }
}

pub struct RedisModuleDigest {
    ptr: NonNull<raw::RedisModuleDigest>,
}

impl RedisModuleDigest {
    pub unsafe fn new(ptr: NonNull<raw::RedisModuleDigest>) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> NonNull<raw::RedisModuleDigest> {
        self.ptr
    }

    pub fn add_str(&mut self, value: impl AsRef<str>) {
        self.add_bytes(value.as_ref().as_bytes());
    }

    pub fn add_bytes(&mut self, value: impl AsRef<[u8]>) {
        let value = value.as_ref();
        let add_str_buffer = unsafe { raw::RedisModule_DigestAddStringBuffer }.unwrap();
        unsafe { add_str_buffer(self.ptr.as_ptr(), value.as_ptr().cast(), value.len()) }
    }

    pub fn add_u64(&mut self, value: u64) {
        self.add_i64(value as i64);
    }

    pub fn add_i64(&mut self, value: i64) {
        let add_long_long = unsafe { raw::RedisModule_DigestAddLongLong }.unwrap();
        unsafe { add_long_long(self.ptr.as_ptr(), value) }
    }

    pub fn end_sequence(&mut self) {
        let add_end_seq = unsafe { raw::RedisModule_DigestEndSequence }.unwrap();
        unsafe { add_end_seq(self.ptr.as_ptr()) }
    }

    pub fn key(&mut self) -> Result<&str, Utf8Error> {
        let get_key = unsafe { raw::RedisModule_GetKeyNameFromDigest }.unwrap();
        RedisString::from_ptr(unsafe { get_key(self.ptr.as_ptr()) })
    }

    pub fn database(&mut self) -> i32 {
        let get_db = unsafe { raw::RedisModule_GetDbIdFromDigest }.unwrap();
        unsafe { get_db(self.ptr.as_ptr()) }
    }

    pub fn add(&mut self, hash: impl std::hash::Hash) {
        let mut hasher = std::hash::DefaultHasher::new();
        hash.hash(&mut hasher);
        self.add_u64(hasher.finish());
    }
}

pub struct RedisModuleKeyOptCtx {
    ptr: NonNull<raw::RedisModuleKeyOptCtx>,
}

impl RedisModuleKeyOptCtx {
    pub unsafe fn new(ptr: NonNull<raw::RedisModuleKeyOptCtx>) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> NonNull<raw::RedisModuleKeyOptCtx> {
        self.ptr
    }

    pub fn key(&mut self) -> Result<&str, Utf8Error> {
        let get_key = unsafe { raw::RedisModule_GetKeyNameFromOptCtx }.unwrap();
        RedisString::from_ptr(unsafe { get_key(self.ptr.as_ptr()) })
    }

    pub fn to_key(&mut self) -> Result<&str, Utf8Error> {
        let get_key = unsafe { raw::RedisModule_GetToKeyNameFromOptCtx }.unwrap();
        RedisString::from_ptr(unsafe { get_key(self.ptr.as_ptr()) })
    }

    pub fn database(&mut self) -> i32 {
        let get_db = unsafe { raw::RedisModule_GetDbIdFromOptCtx }.unwrap();
        unsafe { get_db(self.ptr.as_ptr()) }
    }

    pub fn to_database(&mut self) -> i32 {
        let get_db = unsafe { raw::RedisModule_GetToDbIdFromOptCtx }.unwrap();
        unsafe { get_db(self.ptr.as_ptr()) }
    }
}
