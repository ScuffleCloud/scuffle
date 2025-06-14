use std::borrow::Borrow;
use std::ffi::{CStr, CString};
use std::ops::Add;
use std::ptr::NonNull;

use redis_module::defrag::DefragContext;
use redis_module::{RedisResult, RedisString, raw};

use crate::data_type::io::{AofRewriteIo, RdbLoadIo, RdbSaveIo, RedisModuleDigest, RedisModuleKeyOptCtx};

#[doc(hidden)]
pub trait CStrConv {
    fn into_cstr(self) -> Option<impl Borrow<std::ffi::CStr>>;
}

impl CStrConv for String {
    fn into_cstr(self) -> Option<impl Borrow<std::ffi::CStr>> {
        std::ffi::CString::new(self).ok()
    }
}

impl CStrConv for &String {
    fn into_cstr(self) -> Option<impl Borrow<std::ffi::CStr>> {
        std::ffi::CString::new(self.as_bytes()).ok()
    }
}

impl CStrConv for &str {
    fn into_cstr(self) -> Option<impl Borrow<std::ffi::CStr>> {
        std::ffi::CString::new(self.as_bytes()).ok()
    }
}

impl CStrConv for CString {
    fn into_cstr(self) -> Option<impl Borrow<std::ffi::CStr>> {
        Some(self)
    }
}

impl CStrConv for &CString {
    fn into_cstr(self) -> Option<impl Borrow<std::ffi::CStr>> {
        Some(self.as_ref())
    }
}

impl CStrConv for &CStr {
    fn into_cstr(self) -> Option<impl Borrow<std::ffi::CStr>> {
        Some(self)
    }
}

impl<'a, T> CStrConv for &&'a T
where
    &'a T: CStrConv,
{
    fn into_cstr(self) -> Option<impl Borrow<std::ffi::CStr>> {
        (**self).into_cstr()
    }
}

pub type RdbLoadFn<T> = fn(&mut RdbLoadIo, i32) -> RedisResult<T>;

pub trait RdbLoad<T>: Sized {
    const FN: RdbLoadFn<Self>;

    fn extern_fn() -> raw::RedisModuleTypeLoadFunc {
        unsafe extern "C" fn rdb_load<T, S: RdbLoad<T>>(
            rdb: *mut raw::RedisModuleIO,
            encver: ::std::os::raw::c_int,
        ) -> *mut ::std::os::raw::c_void {
            let Some(rdb) = NonNull::new(rdb) else {
                return std::ptr::null_mut();
            };

            let mut io = unsafe { RdbLoadIo::new(rdb) };
            match S::FN(&mut io, encver) {
                Err(err) => {
                    io.log_warning(err.to_string());
                    std::ptr::null_mut()
                }
                Ok(v) => Box::into_raw(Box::new(v)).cast(),
            }
        }

        Some(rdb_load::<T, Self>)
    }
}

pub type RdbSaveFn<T> = fn(&mut T, &mut RdbSaveIo);

pub trait RdbSave<T>: Sized {
    const FN: RdbSaveFn<Self>;

    fn extern_fn() -> raw::RedisModuleTypeSaveFunc {
        unsafe extern "C" fn rdb_save<T, S: RdbSave<T>>(rdb: *mut raw::RedisModuleIO, value: *mut ::std::os::raw::c_void) {
            let Some(rdb) = NonNull::new(rdb) else {
                return;
            };

            if let Some(value) = unsafe { value.cast::<S>().as_mut() } {
                let mut io = unsafe { RdbSaveIo::new(rdb) };
                S::FN(value, &mut io)
            }
        }

        Some(rdb_save::<T, Self>)
    }
}

pub type AofRewriteFn<T> = fn(&mut T, &mut AofRewriteIo, RedisString) -> RedisResult<()>;

pub trait AofRewrite<T>: Sized {
    const FN: AofRewriteFn<Self>;

    fn extern_fn() -> raw::RedisModuleTypeRewriteFunc {
        unsafe extern "C" fn aof_rewrite<T, S: AofRewrite<T>>(
            aof: *mut raw::RedisModuleIO,
            key: *mut raw::RedisModuleString,
            value: *mut ::std::os::raw::c_void,
        ) {
            let Some(aof) = NonNull::new(aof) else {
                return;
            };

            if let Some(value) = unsafe { value.cast::<S>().as_mut() } {
                let mut io = unsafe { AofRewriteIo::new(aof) };
                if let Err(err) = S::FN(value, &mut io, RedisString::new(None, key)) {
                    io.log_warning(err.to_string());
                }
            }
        }

        Some(aof_rewrite::<T, Self>)
    }
}

pub type MemUsageFn<T> = fn(&T) -> usize;

pub trait MemUsage<T>: Sized {
    const FN: MemUsageFn<Self>;

    fn extern_fn() -> raw::RedisModuleTypeMemUsageFunc {
        unsafe extern "C" fn mem_usage<T, S: MemUsage<T>>(value: *const ::std::os::raw::c_void) -> usize {
            if let Some(value) = unsafe { value.cast::<S>().as_ref() } {
                S::FN(value)
            } else {
                0
            }
        }

        Some(mem_usage::<T, Self>)
    }
}

pub type DigestFn<T> = fn(&mut T, &mut RedisModuleDigest);

pub trait Digest<T>: Sized {
    const FN: DigestFn<Self>;

    fn extern_fn() -> raw::RedisModuleTypeDigestFunc {
        unsafe extern "C" fn digest<T, S: Digest<T>>(
            digest: *mut raw::RedisModuleDigest,
            value: *mut ::std::os::raw::c_void,
        ) {
            let Some(digest) = NonNull::new(digest) else {
                return;
            };

            if let Some(value) = unsafe { value.cast::<S>().as_mut() } {
                let mut io = unsafe { RedisModuleDigest::new(digest) };
                S::FN(value, &mut io)
            }
        }

        Some(digest::<T, Self>)
    }
}

pub type AuxLoadFn = fn(&mut RdbLoadIo, i32, i32) -> RedisResult<i32>;

pub trait AuxLoad<T>: Sized {
    const FN: AuxLoadFn;

    fn extern_fn() -> raw::RedisModuleTypeAuxLoadFunc {
        unsafe extern "C" fn aux_load<T, S: AuxLoad<T>>(rdb: *mut raw::RedisModuleIO, version: i32, when: i32) -> i32 {
            let Some(rdb) = NonNull::new(rdb) else {
                return 0;
            };

            let mut io = unsafe { RdbLoadIo::new(rdb) };
            match S::FN(&mut io, version, when) {
                Ok(v) => v,
                Err(err) => {
                    io.log_warning(err.to_string());
                    0
                }
            }
        }

        Some(aux_load::<T, Self>)
    }
}

pub type AuxSaveFn = fn(&mut RdbSaveIo, i32);

pub trait AuxSave<T>: Sized {
    const FN: AuxSaveFn;

    fn extern_fn() -> raw::RedisModuleTypeAuxSaveFunc {
        unsafe extern "C" fn aux_save<T, S: AuxSave<T>>(rdb: *mut raw::RedisModuleIO, when: i32) {
            let Some(rdb) = NonNull::new(rdb) else {
                return;
            };

            let mut io = unsafe { RdbSaveIo::new(rdb) };
            S::FN(&mut io, when)
        }

        Some(aux_save::<T, Self>)
    }
}

pub type FreeEffortFn<T> = fn(&T, RedisString) -> usize;

pub trait FreeEffort<T>: Sized {
    const FN: FreeEffortFn<Self>;

    fn extern_fn() -> raw::RedisModuleTypeFreeEffortFunc {
        unsafe extern "C" fn free_effort<T, S: FreeEffort<T>>(
            key: *mut raw::RedisModuleString,
            value: *const ::std::os::raw::c_void,
        ) -> usize {
            if let Some(value) = unsafe { value.cast::<S>().as_ref() } {
                S::FN(value, RedisString::new(None, key))
            } else {
                0
            }
        }

        Some(free_effort::<T, Self>)
    }
}

pub type UnlinkFn<T> = fn(&T, RedisString);

pub trait Unlink<T>: Sized {
    const FN: UnlinkFn<Self>;

    fn extern_fn() -> raw::RedisModuleTypeUnlinkFunc {
        unsafe extern "C" fn unlink<T, S: Unlink<T>>(
            key: *mut raw::RedisModuleString,
            value: *const ::std::os::raw::c_void,
        ) {
            if let Some(value) = unsafe { value.cast::<S>().as_ref() } {
                S::FN(value, RedisString::new(None, key))
            }
        }

        Some(unlink::<T, Self>)
    }
}

pub type CopyFn<T> = fn(&T, RedisString, RedisString) -> Option<T>;

pub trait Copy<T>: Sized {
    const FN: CopyFn<Self>;

    fn extern_fn() -> raw::RedisModuleTypeCopyFunc {
        unsafe extern "C" fn copy<T, S: Copy<T>>(
            from_key: *mut raw::RedisModuleString,
            to_key: *mut raw::RedisModuleString,
            value: *const ::std::os::raw::c_void,
        ) -> *mut ::std::os::raw::c_void {
            let value = if let Some(value) = unsafe { value.cast::<S>().as_ref() } {
                S::FN(value, RedisString::new(None, from_key), RedisString::new(None, to_key))
            } else {
                None
            };

            value.map(Box::new).map(Box::into_raw).unwrap_or(std::ptr::null_mut()).cast()
        }

        Some(copy::<T, Self>)
    }
}

pub type DefragFn<T> = fn(&mut T, DefragContext, RedisString) -> i32;

pub trait Defrag<T>: Sized {
    const FN: DefragFn<Self>;

    fn extern_fn() -> raw::RedisModuleTypeDefragFunc {
        unsafe extern "C" fn defrag<T, S: Defrag<T>>(
            ctx: *mut raw::RedisModuleDefragCtx,
            key: *mut raw::RedisModuleString,
            value: *mut *mut ::std::os::raw::c_void,
        ) -> i32 {
            if value.is_null() {
                return 0;
            }

            let value = value as *mut *mut S;

            let ctx = unsafe { DefragContext::new(ctx) };
            unsafe { value.write(ctx.defrag_realloc(*value)) };

            if let Some(value) = unsafe { (*value).as_mut() } {
                S::FN(value, ctx, RedisString::new(None, key))
            } else {
                0
            }
            .add(1)
        }

        Some(defrag::<T, Self>)
    }
}

pub type MemUsage2Fn<T> = fn(&T, &mut RedisModuleKeyOptCtx, usize) -> usize;

pub trait MemUsage2<T>: Sized {
    const FN: MemUsage2Fn<Self>;

    fn extern_fn() -> raw::RedisModuleTypeMemUsageFunc2 {
        unsafe extern "C" fn mem_usage2<T, S: MemUsage2<T>>(
            ctx: *mut raw::RedisModuleKeyOptCtx,
            value: *const ::std::os::raw::c_void,
            sample_size: usize,
        ) -> usize {
            let Some(ctx) = NonNull::new(ctx) else {
                return 0;
            };

            if let Some(value) = unsafe { value.cast::<S>().as_ref() } {
                let mut io = unsafe { RedisModuleKeyOptCtx::new(ctx) };
                S::FN(value, &mut io, sample_size)
            } else {
                0
            }
        }

        Some(mem_usage2::<T, Self>)
    }
}

pub type FreeEffort2Fn<T> = fn(&T, &mut RedisModuleKeyOptCtx) -> usize;

pub trait FreeEffort2<T>: Sized {
    const FN: FreeEffort2Fn<Self>;

    fn extern_fn() -> raw::RedisModuleTypeFreeEffortFunc2 {
        unsafe extern "C" fn free_effort2<T, S: FreeEffort2<T>>(
            ctx: *mut raw::RedisModuleKeyOptCtx,
            value: *const ::std::os::raw::c_void,
        ) -> usize {
            let Some(ctx) = NonNull::new(ctx) else {
                return 0;
            };

            if let Some(value) = unsafe { value.cast::<S>().as_ref() } {
                let mut io = unsafe { RedisModuleKeyOptCtx::new(ctx) };
                S::FN(value, &mut io)
            } else {
                0
            }
        }

        Some(free_effort2::<T, Self>)
    }
}

pub type Unlink2Fn<T> = fn(&T, &mut RedisModuleKeyOptCtx);

pub trait Unlink2<T>: Sized {
    const FN: Unlink2Fn<Self>;

    fn extern_fn() -> raw::RedisModuleTypeUnlinkFunc2 {
        unsafe extern "C" fn unlink2<T, S: Unlink2<T>>(
            ctx: *mut raw::RedisModuleKeyOptCtx,
            value: *const ::std::os::raw::c_void,
        ) {
            let Some(ctx) = NonNull::new(ctx) else {
                return;
            };

            if let Some(value) = unsafe { value.cast::<S>().as_ref() } {
                let mut io = unsafe { RedisModuleKeyOptCtx::new(ctx) };
                S::FN(value, &mut io)
            }
        }

        Some(unlink2::<T, Self>)
    }
}

pub type Copy2Fn<T> = fn(&T, &mut RedisModuleKeyOptCtx) -> Option<T>;

pub trait Copy2<T>: Sized {
    const FN: Copy2Fn<Self>;

    fn extern_fn() -> raw::RedisModuleTypeCopyFunc2 {
        unsafe extern "C" fn copy2<T, S: Copy2<T>>(
            ctx: *mut raw::RedisModuleKeyOptCtx,
            value: *const ::std::os::raw::c_void,
        ) -> *mut ::std::os::raw::c_void {
            let Some(ctx) = NonNull::new(ctx) else {
                return std::ptr::null_mut();
            };

            let value = if let Some(value) = unsafe { value.cast::<S>().as_ref() } {
                let mut io = unsafe { RedisModuleKeyOptCtx::new(ctx) };
                S::FN(value, &mut io)
            } else {
                None
            };

            value.map(Box::new).map(Box::into_raw).unwrap_or(std::ptr::null_mut()).cast()
        }

        Some(copy2::<T, Self>)
    }
}

pub type AuxSave2Fn = fn(&mut RdbSaveIo, i32);

pub trait AuxSave2<T>: Sized {
    const FN: AuxSave2Fn;

    fn extern_fn() -> raw::RedisModuleTypeAuxSaveFunc {
        unsafe extern "C" fn aux_save2<T, S: AuxSave2<T>>(rdb: *mut raw::RedisModuleIO, when: i32) {
            let Some(rdb) = NonNull::new(rdb) else {
                return;
            };

            let mut io = unsafe { RdbSaveIo::new(rdb) };
            S::FN(&mut io, when)
        }

        Some(aux_save2::<T, Self>)
    }
}
