use std::collections::BTreeSet;
use std::num::NonZeroU64;
use std::time::Duration;

use deepsize::DeepSizeOf;
use fnv::FnvHashMap;
use redis_module::key::RedisKeyWritable;
use redis_module::logging::RedisLogLevel;
use redis_module::native_types::RedisType;
use redis_module::{
    Context, NextArg, RedisError, RedisModuleIO, RedisModuleString, RedisResult, RedisString, RedisValue, raw,
};
use redis_module_macros::command;

use crate::utils::{self, SetExt, Str, extern_fn};

#[derive(Clone, bincode::Encode, bincode::Decode, deepsize::DeepSizeOf)]
struct Lease {
    key: Str,
    qty: u64,
    expires_at: Option<NonZeroU64>,
    dims: Vec<Str>,
}

impl Lease {
    fn restore_args(&self) -> impl Iterator<Item = String> {
        [
            self.key.to_string(),
            self.qty.to_string(),
            if let Some(expiry) = self.expires_at {
                expiry.to_string()
            } else {
                "-1".to_string()
            },
        ]
        .into_iter()
        .chain(self.dims.iter().map(|dim| dim.to_string()))
    }
}

#[derive(Default, Clone, deepsize::DeepSizeOf)]
struct Quota {
    leases: FnvHashMap<Str, Lease>,
    dim_count: FnvHashMap<Str, u64>,
    expiry_timeouts: BTreeSet<(NonZeroU64, Str)>,
}

impl bincode::Encode for Quota {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        let length = self.leases.len();

        (length as u64).encode(encoder)?;

        for lease in self.leases.values() {
            lease.encode(encoder)?;
        }

        Ok(())
    }
}

impl<C> bincode::Decode<C> for Quota {
    fn decode<D: bincode::de::Decoder<Context = C>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let length = u64::decode(decoder)?;

        let mut quota = Self::default();
        let now = utils::now();

        for _ in 0..length {
            let lease = Lease::decode(decoder)?;
            if lease.expires_at.is_some_and(|expiry_ts| expiry_ts < now) {
                continue;
            }

            quota.insert_lease(lease);
        }

        Ok(quota)
    }
}

const QUOTA_TYPE_VERSION: i32 = 0;

pub static QUOTA_TYPE: RedisType = RedisType::new(
    "scufquota",
    QUOTA_TYPE_VERSION,
    raw::RedisModuleTypeMethods {
        version: raw::REDISMODULE_TYPE_METHOD_VERSION as u64,
        rdb_load: Some(extern_fn!(|module: *mut RedisModuleIO, version: i32| {
            if version != QUOTA_TYPE_VERSION {
                return std::ptr::null_mut();
            }

            let Ok(buffer) = raw::load_string_buffer(module) else {
                return std::ptr::null_mut();
            };

            let quota: Quota = match bincode::decode_from_slice(buffer.as_ref(), bincode::config::standard()) {
                Ok((v, _)) => v,
                Err(_) => return std::ptr::null_mut(),
            };

            if quota.leases.is_empty() {
                std::ptr::null_mut()
            } else {
                Box::into_raw(Box::new(quota)).cast::<libc::c_void>()
            }
        })),
        rdb_save: Some(extern_fn!(|module: *mut RedisModuleIO, value: *mut libc::c_void| {
            let quota = unsafe { value.cast::<Quota>().as_mut().expect("null pointer") };
            quota.gc(utils::now());
            if !quota.leases.is_empty() {
                let Ok(buf) = bincode::encode_to_vec(&*quota, bincode::config::standard()) else {
                    return;
                };

                raw::save_slice(module, &buf);
            }
        })),
        aof_rewrite: Some(extern_fn!(|module: *mut RedisModuleIO,
                                      key: *mut RedisModuleString,
                                      value: *mut libc::c_void| {
            let key = RedisString::create_from_slice(std::ptr::null_mut(), RedisString::string_as_slice(key));

            let quota = unsafe { value.cast::<Quota>().as_mut().expect("null pointer") };
            quota.gc(utils::now());
            if quota.leases.is_empty() {
                return;
            }
            let emit_aof = unsafe { raw::RedisModule_EmitAOF };
            let Some(emit_aof) = emit_aof else {
                return;
            };

            for lease in quota.leases.values() {
                let restore_args = lease
                    .restore_args()
                    .map(|a| RedisString::create_from_slice(std::ptr::null_mut(), a.as_bytes()))
                    .collect::<Vec<_>>();
                let mut inner_args: Vec<_> = std::iter::once(key.inner)
                    .chain(restore_args.iter().map(|s| s.inner))
                    .collect();

                redis_module::logging::log_io_error(module, RedisLogLevel::Warning, "calling emit_aof");

                unsafe {
                    emit_aof(
                        module,
                        c"quota.restore".as_ptr(),
                        raw::FMT,
                        inner_args.as_mut_ptr(),
                        inner_args.len(),
                    )
                };
            }
        })),
        free: Some(extern_fn!(|ptr: *mut libc::c_void| {
            drop(unsafe { Box::from_raw(ptr.cast::<Quota>()) });
        })),
        mem_usage: Some(extern_fn!(|ptr: *const libc::c_void| {
            let quota = unsafe { ptr.cast::<Quota>().as_ref().expect("null pointer") };

            quota.deep_size_of()
        })),
        digest: None,
        aux_load: None,
        aux_save: None,
        aux_save2: None,
        aux_save_triggers: 0,
        free_effort: None,
        unlink: None,
        copy: Some(extern_fn!(|_fromkey: *mut RedisModuleString,
                               _tokey: *mut RedisModuleString,
                               value: *const libc::c_void| {
            let quota = unsafe { value.cast::<Quota>().as_ref().expect("null pointer") };
            Box::into_raw(Box::new(quota.clone())).cast::<libc::c_void>()
        })),
        copy2: None,
        defrag: None,
        free_effort2: None,
        mem_usage2: None,
        unlink2: None,
    },
);

enum QuotaMutRef<'a> {
    Owned(Quota),
    Ref(&'a mut Quota),
}

impl std::ops::Deref for QuotaMutRef<'_> {
    type Target = Quota;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(o) => o,
            Self::Ref(r) => r,
        }
    }
}

impl std::ops::DerefMut for QuotaMutRef<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Owned(o) => o,
            Self::Ref(r) => r,
        }
    }
}

fn quota_scope(
    key: &RedisKeyWritable,
    mut quota: QuotaMutRef<'_>,
    op: impl FnOnce(&mut Quota, NonZeroU64) -> RedisResult,
) -> RedisResult {
    let now = utils::now();
    quota.gc(now);
    let ret = op(&mut quota, now);

    if quota.leases.is_empty() || quota.expires_at().is_some_and(|expiry| expiry < now) {
        if !key.is_empty() {
            key.delete()?;
        }
    } else {
        let expires = quota.expires_at();
        if let QuotaMutRef::Owned(o) = quota {
            key.set_value(&QUOTA_TYPE, o)?;
        }

        if let Some(seconds) = expires.map(|ex| ex.get() - now.get()) {
            key.set_expire(Duration::from_secs(seconds))?;
        } else {
            key.remove_expire()?;
        }
    }

    ret
}

#[command({
    name: "quota.lease",
    summary: "aquire a new lease",
    complexity: "O(log(N) + D): Where N is the number of pending leases and D is the number of dimentions.",
    flags: [Write, DenyOOM, Fast],
    arity: -7,
    key_spec: [
        {
            flags: [ReadWrite, Access, Update],
            begin_search: Index({ index: 1 }),
            find_keys: Range({ last_key: 0, steps: 1, limit: 0 }),
        }
    ]
})]
fn quota_lease(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1).peekable();
    let resource = args.next_arg()?;
    let key = ctx.open_key_writable(&resource);

    let quota = if let Some(value) = key.get_value::<Quota>(&QUOTA_TYPE)? {
        QuotaMutRef::Ref(value)
    } else {
        QuotaMutRef::Owned(Quota::default())
    };

    quota_scope(&key, quota, |quota, now| {
        let key = Str::from(args.next_str()?);
        if quota.leases.contains_key(&key) {
            return Err(RedisError::Str("LEASE_ALREADY_EXISTS"));
        }

        let qty = args.next_u64()?;
        let expires_at = args.next_i64()?;
        let expires_at = if expires_at < 0 {
            None
        } else if let Some(expiry_ts) = NonZeroU64::new(expires_at as u64) {
            let Some(expires_at) = expiry_ts.checked_add(now.get()) else {
                return Err(RedisError::Str("TTL_EXCEEDS_MAX_TIMESTAMP"));
            };
            Some(expires_at)
        } else {
            return Ok(RedisValue::Integer(0));
        };

        let mut dims = Vec::new();
        while args.peek().is_some() {
            let dim = args.next_str()?;
            let limit = args.next_u64()?;
            if quota.dim_count.get(dim).is_some_and(|count| *count + qty > limit) {
                return Err(RedisError::String(format!("QUOTA_EXCEEDED:{dim}")));
            }

            dims.push(Str::from(dim));
        }

        let lease = Lease {
            key,
            dims,
            expires_at,
            qty,
        };

        let args = lease.restore_args().map(|arg| ctx.create_string(arg)).collect::<Vec<_>>();
        let args = std::iter::once(&resource).chain(args.iter()).collect::<Vec<_>>();

        ctx.replicate("quota.restore", args.as_slice());
        quota.insert_lease(lease);

        Ok(RedisValue::Integer(1))
    })
}

#[command({
    name: "quota.restore",
    summary: "restore a lease (used by aof)",
    complexity: "O(log(N) + D): Where N is the number of pending leases and D is the number of dimentions.",
    flags: [Write, DenyOOM, Fast],
    arity: -7,
    key_spec: [
        {
            flags: [ReadWrite, Access, Update],
            begin_search: Index({ index: 1 }),
            find_keys: Range({ last_key: 0, steps: 1, limit: 0 }),
        }
    ]
})]
fn quota_restore(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1).peekable();
    let resource = args.next_arg()?;
    let key = ctx.open_key_writable(&resource);

    let quota = if let Some(value) = key.get_value::<Quota>(&QUOTA_TYPE)? {
        QuotaMutRef::Ref(value)
    } else {
        QuotaMutRef::Owned(Quota::default())
    };

    quota_scope(&key, quota, |quota, now| {
        let key = Str::from(args.next_str()?);
        quota.remove_lease(&key);

        let qty = args.next_u64()?;
        let expires_at = if let Some(expires_at) = NonZeroU64::new(args.next_u64()?) {
            if expires_at < now {
                return Ok(RedisValue::Integer(0));
            }

            Some(expires_at)
        } else {
            None
        };

        let mut dims = Vec::new();
        while args.peek().is_some() {
            dims.push(Str::from(args.next_str()?));
        }

        quota.insert_lease(Lease {
            dims,
            expires_at,
            key,
            qty,
        });

        Ok(RedisValue::Integer(1))
    })
}

#[command({
    name: "quota.renew",
    summary: "renew a previously allocated lease",
    complexity: "O(log(N)): Where N is the number of pending leases.",
    flags: [Write, DenyOOM],
    arity: 4,
    key_spec: [
        {
            flags: [ReadWrite, Access, Update],
            begin_search: Index({ index: 1 }),
            find_keys: Range({ last_key: 1, steps: 1, limit: 1 }),
        }
    ]
})]
fn quota_renew(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1).peekable();
    let resource = args.next_arg()?;
    let key = ctx.open_key_writable(&resource);

    let Some(quota) = key.get_value::<Quota>(&QUOTA_TYPE)? else {
        return Ok(RedisValue::Integer(0));
    };

    quota_scope(&key, QuotaMutRef::Ref(quota), |quota, now| {
        let lease_id = args.next_str()?;
        match NonZeroU64::new(args.next_u64()?).map(|ts| ts.checked_add(now.get())) {
            Some(Some(expire_ts)) => {
                if quota.renew_lease(lease_id, now, Some(expire_ts)) {
                    ctx.replicate(
                        "quota.renew_at",
                        &[resource.as_slice(), lease_id.as_bytes(), expire_ts.to_string().as_bytes()],
                    );

                    Ok(RedisValue::Integer(1))
                } else {
                    Ok(RedisValue::Integer(0))
                }
            }
            Some(None) => Err(RedisError::Str("TTL_EXCEEDS_MAX_TIMESTAMP")),
            None => {
                if quota.remove_lease(lease_id) {
                    ctx.replicate("quota.free", &[resource.as_slice(), lease_id.as_bytes()]);

                    Ok(RedisValue::Integer(1))
                } else {
                    Ok(RedisValue::Integer(0))
                }
            }
        }
    })
}

#[command({
    name: "quota.renew_at",
    summary: "renew a previously allocated lease",
    complexity: "O(log(N)): Where N is the number of pending leases.",
    flags: [Write, DenyOOM],
    arity: 4,
    key_spec: [
        {
            flags: [ReadWrite, Access, Update],
            begin_search: Index({ index: 1 }),
            find_keys: Range({ last_key: 1, steps: 1, limit: 1 }),
        }
    ]
})]
fn quota_renew_at(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1).peekable();
    let resource = args.next_arg()?;
    let key = ctx.open_key_writable(&resource);

    let Some(quota) = key.get_value::<Quota>(&QUOTA_TYPE)? else {
        return Ok(RedisValue::Integer(0));
    };

    quota_scope(&key, QuotaMutRef::Ref(quota), |quota, now| {
        let lease_id = args.next_str()?;

        match NonZeroU64::new(args.next_u64()?) {
            Some(expires_at) if expires_at > now => {
                if quota.renew_lease(lease_id, now, Some(expires_at)) {
                    ctx.replicate_verbatim();
                    Ok(RedisValue::Integer(1))
                } else {
                    Ok(RedisValue::Integer(0))
                }
            }
            _ => {
                if quota.remove_lease(lease_id) {
                    ctx.replicate("quota.free", &[resource.as_slice(), lease_id.as_bytes()]);

                    Ok(RedisValue::Integer(1))
                } else {
                    Ok(RedisValue::Integer(0))
                }
            }
        }
    })
}

#[command({
    name: "quota.commit",
    summary: "commits a previously allocated lease",
    complexity: "O(log(N)): Where N is the number of pending leases.",
    flags: [Write, DenyOOM],
    arity: 3,
    key_spec: [
        {
            flags: [ReadWrite, Access, Update],
            begin_search: Index({ index: 1 }),
            find_keys: Range({ last_key: 1, steps: 1, limit: 1 }),
        }
    ]
})]
fn quota_commit(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1).peekable();
    let resource = args.next_arg()?;
    let key = ctx.open_key_writable(&resource);

    let Some(quota) = key.get_value::<Quota>(&QUOTA_TYPE)? else {
        return Ok(RedisValue::Integer(0));
    };

    quota_scope(&key, QuotaMutRef::Ref(quota), |quota, now| {
        let lease_id = args.next_str()?;

        if quota.renew_lease(lease_id, now, None) {
            ctx.replicate_verbatim();
            Ok(RedisValue::Integer(1))
        } else {
            Ok(RedisValue::Integer(0))
        }
    })
}

#[command({
    name: "quota.free",
    summary: "free a previouly allocated lease",
    complexity: "O(log(N) + D): Where N is the number of pending leases and D is the number of dimentions.",
    flags: [Write, DenyOOM],
    arity: 3,
    key_spec: [
        {
            flags: [ReadWrite, Access, Update],
            begin_search: Index({ index: 1 }),
            find_keys: Range({ last_key: 1, steps: 1, limit: 1 }),
        }
    ]
})]
fn quota_free(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1).peekable();
    let resource = args.next_arg()?;
    let key = ctx.open_key_writable(&resource);

    let Some(quota) = key.get_value::<Quota>(&QUOTA_TYPE)? else {
        return Ok(RedisValue::Integer(0));
    };

    quota_scope(&key, QuotaMutRef::Ref(quota), |quota, _| {
        let lease_id = args.next_str()?;
        ctx.replicate_verbatim();

        if quota.remove_lease(lease_id) {
            Ok(RedisValue::Integer(1))
        } else {
            Ok(RedisValue::Integer(0))
        }
    })
}

impl Quota {
    fn gc(&mut self, now_ts: NonZeroU64) {
        while let Some((_, lease_id)) = self.expiry_timeouts.pop_first_if(|(timeout, _)| *timeout < now_ts) {
            let Some(lease) = self.leases.remove(&lease_id) else {
                continue;
            };

            for dim in lease.dims {
                if let Some(count) = self.dim_count.get_mut(&dim) {
                    *count = count.saturating_sub(lease.qty);
                    if *count == 0 {
                        self.dim_count.remove(&dim);
                    }
                }
            }
        }
    }

    fn expires_at(&self) -> Option<NonZeroU64> {
        self.expiry_timeouts.last().map(|(ts, _)| *ts)
    }

    fn remove_lease(&mut self, id: &str) -> bool {
        let Some(lease) = self.leases.remove(id) else {
            return false;
        };

        for dim in lease.dims {
            if let Some(count) = self.dim_count.get_mut(&dim) {
                *count = count.saturating_sub(lease.qty);
                if *count == 0 {
                    self.dim_count.remove(&dim);
                }
            }
        }

        if let Some(expiry_ts) = lease.expires_at {
            self.expiry_timeouts.remove(&(expiry_ts, lease.key));
        }

        true
    }

    fn insert_lease(&mut self, lease: Lease) {
        for dim in &lease.dims {
            let count = self.dim_count.entry(dim.clone()).or_default();
            *count += lease.qty;
        }

        if let Some(expires_ts) = lease.expires_at {
            self.expiry_timeouts.insert((expires_ts, lease.key.clone()));
        }

        self.leases.insert(lease.key.clone(), lease);
    }

    fn renew_lease(&mut self, id: &str, now: NonZeroU64, expires_at: Option<NonZeroU64>) -> bool {
        let Some(lease) = self.leases.get_mut(id) else {
            return false;
        };

        if expires_at.is_some_and(|expires_at| expires_at < now) {
            return self.remove_lease(id);
        }

        if lease.expires_at == expires_at {
            return false;
        }

        if let Some(expires_at) = lease.expires_at {
            self.expiry_timeouts.remove(&(expires_at, lease.key.clone()));
        }

        if let Some(expires_at) = expires_at {
            self.expiry_timeouts.insert((expires_at, lease.key.clone()));
        }

        lease.expires_at = expires_at;

        true
    }
}
