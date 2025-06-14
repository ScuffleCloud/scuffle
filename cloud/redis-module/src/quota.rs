use std::collections::BTreeSet;
use std::num::NonZeroU64;
use std::time::Duration;

use deepsize::DeepSizeOf;
use fnv::FnvHashMap;
use redis_module::key::RedisKeyWritable;
use redis_module::{Context, NextArg, RedisError, RedisResult, RedisString, RedisValue};
use redis_module_ext::data_type::{AofRewriteIo, RdbLoadIo, RdbSaveIo};
use redis_module_ext::prelude::{IoLoggingExt, RedisKeyWritableExt};
use redis_module_ext::{CommandArgs, redis_command, redis_data_type, redis_module};

use crate::utils::{self, SetExt, Str, now};

#[derive(Clone, bincode::Encode, bincode::Decode, deepsize::DeepSizeOf)]
struct Lease {
    id: Str,
    qty: u64,
    expires_at: Option<NonZeroU64>,
    dims: Vec<Str>,
}

impl Lease {
    fn restore_args(&self) -> impl Iterator<Item = String> {
        [
            self.id.to_string(),
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

const QUOTA_TYPE_VERSION: i32 = 0;

#[redis_data_type(
    name = "scufquota",
    version = QUOTA_TYPE_VERSION,
    methods(
        rdb_load,
        rdb_save,
        aof_rewrite,
        mem_usage,
    )
)]
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

const BINCODE_CONFIG: bincode::config::Configuration = bincode::config::standard();

impl Quota {
    fn rdb_load(rdb: &mut RdbLoadIo, version: i32) -> RedisResult<Self> {
        if version != QUOTA_TYPE_VERSION {
            return Err(RedisError::String(format!(
                "unsupported quota type version expected: {QUOTA_TYPE_VERSION}, actual: {version}"
            )));
        }

        let buf = rdb.load_string_buffer()?;
        let quota: Quota = match bincode::decode_from_slice(buf.as_ref(), BINCODE_CONFIG) {
            Ok((v, _)) => v,
            Err(err) => return Err(RedisError::String(format!("failed to decode quota: {err}"))),
        };

        Ok(quota)
    }

    fn rdb_save(&mut self, rdb: &mut RdbSaveIo) {
        self.gc(now());

        match bincode::encode_to_vec(&*self, BINCODE_CONFIG) {
            Ok(buf) => {
                rdb.save_slice(buf);
            }
            Err(err) => {
                rdb.log_warning(format!("failed to encode: {err}"));
            }
        }
    }

    fn aof_rewrite(&mut self, aof: &mut AofRewriteIo, key: RedisString) -> RedisResult<()> {
        self.gc(utils::now());
        if self.leases.is_empty() {
            return Ok(());
        }

        for lease in self.leases.values() {
            aof.emit_command(c"quota.restore")
                .arg(&*key)
                .args(lease.restore_args())
                .dispatch()?;
        }

        Ok(())
    }

    fn mem_usage(&self) -> usize {
        self.deep_size_of()
    }
}

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
            key.set(o)?;
        }

        if let Some(seconds) = expires.map(|ex| ex.get() - now.get()) {
            key.set_expire(Duration::from_secs(seconds))?;
        } else {
            key.remove_expire()?;
        }
    }

    ret
}

#[redis_command(
    name = "quota.lease",
    summary = "aquire a new lease",
    complexity = "O(log(N) + D): Where N is the number of pending leases and D is the number of dimentions.",
    flags(Write, DenyOOM, Fast),
    arity = -7,
    key_spec(
        flags(ReadWrite, Access, Update),
        begin_search(index = 1),
        find_keys(range(last_key = 1, steps = 1, limit = 1))
    ),
    arg(
        name = "KEY",
        kind = Key,
        key_spec_index = 0,
    ),
    arg(
        name = "ID",
        kind = String,
    ),
    arg(
        name = "QTY",
        kind = Integer,
    ),
    arg(
        name = "TTL",
        kind = Integer,
    ),
    arg(
        name = "dim_limit",
        kind = Block,
        flags = 2,
        arg(
            name = "DIM",
            kind = String,
        ),
        arg(
            name = "LIMIT",
            kind = Integer,
        ),
    )
)]
fn quota_lease(ctx: &Context, args: CommandArgs) -> RedisResult {
    let mut args = args.into_iter().skip(1).peekable();
    let resource = args.next_arg()?;
    let key = ctx.open_key_writable(&resource);

    let quota = if let Some(value) = key.get::<Quota>()? {
        QuotaMutRef::Ref(value)
    } else {
        QuotaMutRef::Owned(Quota::default())
    };

    quota_scope(&key, quota, |quota, now| {
        let lease_id = Str::from(args.next_str()?);
        if quota.leases.contains_key(&lease_id) {
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
            id: lease_id,
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

#[redis_command(
    name = "quota.restore",
    flags(Internal),
    arity = -7,
    key_spec(
        flags(ReadWrite, Access, Update),
        begin_search(index = 1),
        find_keys(range(last_key = 1, steps = 1, limit = 1)),
    )
)]
fn quota_restore(ctx: &Context, args: CommandArgs) -> RedisResult {
    let mut args = args.into_iter().skip(1).peekable();
    let resource = args.next_arg()?;
    let key = ctx.open_key_writable(&resource);

    let quota = if let Some(value) = key.get::<Quota>()? {
        QuotaMutRef::Ref(value)
    } else {
        QuotaMutRef::Owned(Quota::default())
    };

    quota_scope(&key, quota, |quota, now| {
        let lease_id = Str::from(args.next_str()?);
        quota.remove_lease(&lease_id);

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
            id: lease_id,
            qty,
        });

        Ok(RedisValue::Integer(1))
    })
}

#[redis_command(
    name = "quota.renew",
    summary = "renew a previously allocated lease",
    complexity = "O(log(N)): Where N is the number of pending leases.",
    flags(Write, DenyOOM),
    arity = 4,
    key_spec(
        flags(ReadWrite, Access, Update),
        begin_search(index = 1),
        find_keys(range(last_key = 1, steps = 1, limit = 1)),
    ),
    arg(
        name = "KEY",
        kind = Key,
        key_spec_index = 0,
    ),
    arg(
        name = "ID",
        kind = String,
    ),
    arg(
        name = "TTL",
        kind = Integer,
    ),
)]
fn quota_renew(ctx: &Context, args: CommandArgs) -> RedisResult {
    let mut args = args.into_iter().skip(1).peekable();
    let resource = args.next_arg()?;
    let key = ctx.open_key_writable(&resource);

    let Some(quota) = key.get::<Quota>()? else {
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

#[redis_command(
    name = "quota.renew_at",
    summary = "renew a previously allocated lease",
    complexity = "O(log(N)): Where N is the number of pending leases.",
    flags(Write, DenyOOM),
    arity = 4,
    key_spec(
        flags(ReadWrite, Access, Update),
        begin_search(index = 1),
        find_keys(range(last_key = 1, steps = 1, limit = 1)),
    ),
    arg(
        name = "KEY",
        kind = Key,
        key_spec_index = 0,
    ),
    arg(
        name = "ID",
        kind = String,
    ),
    arg(
        name = "EXPIRY",
        kind = Integer,
    ),
)]
fn quota_renew_at(ctx: &Context, args: CommandArgs) -> RedisResult {
    let mut args = args.into_iter().skip(1).peekable();
    let resource = args.next_arg()?;
    let key = ctx.open_key_writable(&resource);

    let Some(quota) = key.get::<Quota>()? else {
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

#[redis_command(
    name = "quota.commit",
    summary = "commits a previously allocated lease",
    complexity = "O(log(N)): Where N is the number of pending leases.",
    flags(Write, DenyOOM),
    arity = 3,
    key_spec(
        flags(ReadWrite, Access, Update),
        begin_search(index = 1),
        find_keys(range(last_key = 1, steps = 1, limit = 1)),
    ),
    arg(
        name = "KEY",
        kind = Key,
        key_spec_index = 0,
    ),
    arg(
        name = "ID",
        kind = String,
    ),
)]
fn quota_commit(ctx: &Context, args: CommandArgs) -> RedisResult {
    let mut args = args.into_iter().skip(1).peekable();
    let resource = args.next_arg()?;
    let key = ctx.open_key_writable(&resource);

    let Some(quota) = key.get::<Quota>()? else {
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

#[redis_command(
    name = "quota.free",
    summary = "free a previouly allocated lease",
    complexity = "O(log(N) + D): Where N is the number of pending leases and D is the number of dimentions.",
    flags(Write, DenyOOM),
    arity = 3,
    key_spec(
        flags(ReadWrite, Access, Update),
        begin_search(index = 1),
        find_keys(range(last_key = 1, steps = 1, limit = 1)),
    ),
    arg(
        name = "KEY",
        kind = Key,
        key_spec_index = 0,
    ),
    arg(
        name = "ID",
        kind = String,
    ),
)]
fn quota_free(ctx: &Context, args: CommandArgs) -> RedisResult {
    let mut args = args.into_iter().skip(1).peekable();
    let resource = args.next_arg()?;
    let key = ctx.open_key_writable(&resource);

    let Some(quota) = key.get::<Quota>()? else {
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
            self.expiry_timeouts.remove(&(expiry_ts, lease.id));
        }

        true
    }

    fn insert_lease(&mut self, lease: Lease) {
        for dim in &lease.dims {
            let count = self.dim_count.entry(dim.clone()).or_default();
            *count += lease.qty;
        }

        if let Some(expires_ts) = lease.expires_at {
            self.expiry_timeouts.insert((expires_ts, lease.id.clone()));
        }

        self.leases.insert(lease.id.clone(), lease);
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
            self.expiry_timeouts.remove(&(expires_at, lease.id.clone()));
        }

        if let Some(expires_at) = expires_at {
            self.expiry_timeouts.insert((expires_at, lease.id.clone()));
        }

        lease.expires_at = expires_at;

        true
    }
}

#[redis_module(
    name = "quota",
    version = 1,
    types(Quota),
    commands(quota_free, quota_commit, quota_renew_at, quota_renew, quota_restore, quota_lease,)
)]
pub struct QuotaModule;
