use std::borrow::{Borrow, Cow};
use std::ffi::{CStr, CString};
use std::fmt::Write;
use std::marker::PhantomData;
use std::ptr::NonNull;

use redis_module::raw;

pub trait RedisCommand: Sized {
    const NAME: &'static CStr;

    fn flags(ctx: &redis_module::Context) -> Vec<CommandFlag>;
    fn command_info(ctx: &redis_module::Context) -> RedisModuleCommandInfo;
    fn invoke(ctx: &redis_module::Context, args: CommandArgs) -> redis_module::RedisResult;

    fn invoke_raw(ctx: *mut raw::RedisModuleCtx, argv: *mut *mut raw::RedisModuleString, argc: i32) -> i32 {
        let mut wctx = redis_module::Context::new(ctx);
        let resp = Self::invoke(&mut wctx, CommandArgs::new(ctx, argv, argc as usize));
        wctx.reply(resp) as i32
    }

    fn register(ctx: &redis_module::Context) -> redis_module::RedisResult<()> {
        unsafe extern "C" fn callback<C: RedisCommand>(
            ctx: *mut raw::RedisModuleCtx,
            argv: *mut *mut raw::RedisModuleString,
            argc: i32,
        ) -> i32 {
            C::invoke_raw(ctx, argv, argc)
        }

        let name = Self::NAME.to_string_lossy();
        let flags = fmtools::fmt(|f| {
            let mut first = true;
            for flag in Self::flags(ctx) {
                if !first {
                    f.write_char(' ')?;
                }
                first = false;

                f.write_str(flag.as_str())?;
            }
            Ok(())
        })
        .to_string();

        let flags = CString::new(flags).unwrap();

        if unsafe {
            redis_module::RedisModule_CreateCommand.unwrap()(
                ctx.ctx,
                Self::NAME.as_ptr(),
                Some(callback::<Self>),
                flags.as_ptr(),
                0,
                0,
                0,
            )
        } == raw::Status::Err as i32
        {
            return Err(redis_module::RedisError::String(format!("Failed register command {name}.",)));
        }

        // Register the extra data of the command
        let command = unsafe { raw::RedisModule_GetCommand.unwrap()(ctx.ctx, Self::NAME.as_ptr()) };

        if command.is_null() {
            return Err(redis_module::RedisError::String(format!(
                "Failed finding command {name} after registration.",
            )));
        }

        let command_info = Self::command_info(ctx);
        let mut ffi = command_info.as_ffi();

        if unsafe { raw::RedisModule_SetCommandInfo.unwrap()(command, ffi.ffi_mut()) } == raw::Status::Err as i32 {
            return Err(redis_module::RedisError::String(format!(
                "Failed setting info for command {name}.",
            )));
        }

        Ok(())
    }
}

pub struct CommandArgs {
    ctx: Option<NonNull<raw::RedisModuleCtx>>,
    argv: *mut *mut raw::RedisModuleString,
    argc: usize,
}

impl CommandArgs {
    fn new(ctx: *mut raw::RedisModuleCtx, argv: *mut *mut raw::RedisModuleString, argc: usize) -> Self {
        CommandArgs {
            ctx: NonNull::new(ctx),
            argv,
            argc,
        }
    }
}

impl Iterator for CommandArgs {
    type Item = redis_module::RedisString;

    fn next(&mut self) -> Option<Self::Item> {
        if self.argc == 0 {
            None
        } else {
            self.argc -= 1;
            let arg = unsafe { *self.argv };
            self.argv = unsafe { self.argv.add(1) };
            Some(redis_module::RedisString::new(self.ctx, arg))
        }
    }
}

pub struct RedisModuleCommandInfo {
    pub summary: Option<Cow<'static, CStr>>,
    pub complexity: Option<Cow<'static, CStr>>,
    pub since: Option<Cow<'static, CStr>>,
    pub history: Vec<RedisModuleCommandHistoryEntry>,
    pub tips: Option<Cow<'static, CStr>>,
    pub arity: i32,
    pub key_specs: Vec<RedisModuleCommandKeySpec>,
    pub args: Vec<RedisModuleCommandArg>,
}

pub struct RedisModuleCommandHistoryEntry {
    pub since: Option<Cow<'static, CStr>>,
    pub changes: Option<Cow<'static, CStr>>,
}

pub struct RedisModuleCommandKeySpec {
    pub notes: Option<Cow<'static, CStr>>,
    pub flags: Vec<KeySpecFlag>,
    pub begin_search: KeySpecBeginSearch,
    pub find_keys: Option<KeySpecFindKeys>,
}

pub enum KeySpecBeginSearch {
    Index(i32),
    Keyword {
        keyword: Cow<'static, CStr>,
        start_from: i32,
    },
}

pub enum KeySpecFindKeys {
    Range {
        last_key: i32,
        key_step: i32,
        limit: i32,
    },
    Keynum {
        keynum_idx: i32,
        first_key: i32,
        key_step: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(u32)]
pub enum RedisModuleCommandArgKind {
    /// String argument.
    String = raw::RedisModuleCommandArgType_REDISMODULE_ARG_TYPE_STRING,
    /// Integer argument.
    Integer = raw::RedisModuleCommandArgType_REDISMODULE_ARG_TYPE_INTEGER,
    /// Double-precision float argument.
    Double = raw::RedisModuleCommandArgType_REDISMODULE_ARG_TYPE_DOUBLE,
    /// String argument representing a keyname.
    Key = raw::RedisModuleCommandArgType_REDISMODULE_ARG_TYPE_KEY,
    /// String, but regex pattern.
    Pattern = raw::RedisModuleCommandArgType_REDISMODULE_ARG_TYPE_PATTERN,
    /// Integer, but Unix timestamp.
    UnixTime = raw::RedisModuleCommandArgType_REDISMODULE_ARG_TYPE_UNIX_TIME,
    /// Argument doesn't have a placeholder. It's just a token without a value. Example: the KEEPTTL option of the SET command.
    PureToken = raw::RedisModuleCommandArgType_REDISMODULE_ARG_TYPE_PURE_TOKEN,
    /// Used when the user can choose only one of a few sub-arguments. Requires subargs. Example: the NX and XX options of SET.
    OneOf = raw::RedisModuleCommandArgType_REDISMODULE_ARG_TYPE_ONEOF,
    /// Used when one wants to group together several sub-arguments, usually to apply something on all of them,
    /// like making the entire group "optional". Requires subargs. Example: the LIMIT offset count parameters in ZRANGE.
    Block = raw::RedisModuleCommandArgType_REDISMODULE_ARG_TYPE_BLOCK,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(u32)]
pub enum RedisModuleCommandArgFlag {
    /// The argument is optional (like GET in the SET command).
    Optional = raw::REDISMODULE_CMD_ARG_OPTIONAL,
    /// The argument may repeat itself (like key in DEL).
    Multiple = raw::REDISMODULE_CMD_ARG_MULTIPLE,
    /// The argument may repeat itself, and so does its token (like GET pattern in SORT).
    MultipleToken = raw::REDISMODULE_CMD_ARG_MULTIPLE_TOKEN,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum CommandFlag {
    /// The command may modify the data set (it may also read from it).
    Write,

    /// The command returns data from keys but never writes.
    ReadOnly,

    /// The command is an administrative command (may change replication or perform similar tasks).
    Admin,

    /// The command may use additional memory and should be denied during out of memory conditions.
    DenyOOM,

    /// Don't allow this command in Lua scripts.
    DenyScript,

    /// Allow this command while the server is loading data. Only commands not interacting with the data set
    /// should be allowed to run in this mode. If not sure don't use this flag.
    AllowLoading,

    /// The command publishes things on Pub/Sub channels.
    PubSub,

    /// The command may have different outputs even starting from the same input arguments and key values.
    /// Starting from Redis 7.0 this flag has been deprecated. Declaring a command as "random" can be done using
    /// command tips, see https://redis.io/topics/command-tips.
    #[deprecated = "Declaring a command as 'random' can be done using command tips, see https://redis.io/topics/command-tips."]
    Random,

    /// The command is allowed to run on slaves that don't serve stale data. Don't use if you don't know what
    /// this means.
    AllowStale,

    /// Don't propagate the command on monitor. Use this if the command has sensitive data among the arguments.
    NoMonitor,

    /// Don't log this command in the slowlog. Use this if the command has sensitive data among the arguments.
    NoSlowlog,

    /// The command time complexity is not greater than O(log(N)) where N is the size of the collection or
    /// anything else representing the normal scalability issue with the command.
    Fast,

    /// The command implements the interface to return the arguments that are keys. Used when start/stop/step
    /// is not enough because of the command syntax.
    GetkeysApi,

    /// The command should not register in Redis Cluster since is not designed to work with it because, for
    /// example, is unable to report the position of the keys, programmatically creates key names, or any
    /// other reason.
    NoCluster,

    /// This command can be run by an un-authenticated client. Normally this is used by a command that is used
    /// to authenticate a client.
    NoAuth,

    /// This command may generate replication traffic, even though it's not a write command.
    MayReplicate,

    /// All the keys this command may take are optional
    NoMandatoryKeys,

    /// The command has the potential to block the client.
    Blocking,

    /// Permit the command while the server is blocked either by a script or by a slow module command, see
    /// RM_Yield.
    AllowBusy,

    /// The command implements the interface to return the arguments that are channels.
    GetchannelsApi,

    /// Internal command, one that should not be exposed to the user connections.
    /// For example, module commands that are called by the modules, commands that do not perform ACL validations (relying on earlier checks)
    Internal,
}

impl CommandFlag {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Write => "write",
            Self::ReadOnly => "readonly",
            Self::Admin => "admin",
            Self::DenyOOM => "deny-oom",
            Self::DenyScript => "deny-script",
            Self::AllowLoading => "allow-loading",
            Self::PubSub => "pubsub",
            #[allow(deprecated)]
            Self::Random => "random",
            Self::AllowStale => "allow-stale",
            Self::NoMonitor => "no-monitor",
            Self::NoSlowlog => "no-slowlog",
            Self::Fast => "fast",
            Self::GetkeysApi => "getkeys-api",
            Self::NoCluster => "no-cluster",
            Self::NoAuth => "no-auth",
            Self::MayReplicate => "may-replicate",
            Self::NoMandatoryKeys => "no-mandatory-keys",
            Self::Blocking => "blocking",
            Self::AllowBusy => "allow-busy",
            Self::GetchannelsApi => "getchannels-api",
            Self::Internal => "internal",
        }
    }
}

impl std::fmt::Display for CommandFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(u32)]
pub enum KeySpecFlag {
    /// Read-Only. Reads the value of the key, but doesn't necessarily return it.
    ReadOnly = raw::REDISMODULE_CMD_KEY_RO,

    /// Read-Write. Modifies the data stored in the value of the key or its metadata.
    ReadWrite = raw::REDISMODULE_CMD_KEY_RW,

    /// Overwrite. Overwrites the data stored in the value of the key.
    Overwrite = raw::REDISMODULE_CMD_KEY_OW,

    /// Deletes the key.
    Remove = raw::REDISMODULE_CMD_KEY_RM,

    /// Returns, copies or uses the user data from the value of the key.
    Access = raw::REDISMODULE_CMD_KEY_ACCESS,

    /// Updates data to the value, new value may depend on the old value.
    Update = raw::REDISMODULE_CMD_KEY_UPDATE,

    /// Adds data to the value with no chance of modification or deletion of existing data.
    Insert = raw::REDISMODULE_CMD_KEY_INSERT,

    /// Explicitly deletes some content from the value of the key.
    Delete = raw::REDISMODULE_CMD_KEY_DELETE,

    /// The key is not actually a key, but should be routed in cluster mode as if it was a key.
    NotKey = raw::REDISMODULE_CMD_KEY_NOT_KEY,

    /// The keyspec might not point out all the keys it should cover.
    Incomplete = raw::REDISMODULE_CMD_KEY_INCOMPLETE,

    /// Some keys might have different flags depending on arguments.
    VariableFlags = raw::REDISMODULE_CMD_KEY_VARIABLE_FLAGS,
}

fn opt_cstr_ptr<R: AsRef<CStr>>(cstr: impl Borrow<Option<R>>) -> *const std::ffi::c_char {
    cstr.borrow()
        .as_ref()
        .map(|cstr| cstr.as_ref().as_ptr())
        .unwrap_or(std::ptr::null())
}

pub struct RedisModuleCommandArg {
    pub name: Cow<'static, CStr>,
    pub kind: RedisModuleCommandArgKind,
    pub key_spec_idx: i32,
    pub token: Option<Cow<'static, CStr>>,
    pub summary: Option<Cow<'static, CStr>>,
    pub since: Option<Cow<'static, CStr>>,
    pub flags: i32,
    pub deprecated_since: Option<Cow<'static, CStr>>,
    pub sub_args: Vec<RedisModuleCommandArg>,
    pub display_text: Option<Cow<'static, CStr>>,
}

struct RawRedisModuleCommandInfo<'a> {
    ffi: raw::RedisModuleCommandInfo,
    _history_storage: Box<[raw::RedisModuleCommandHistoryEntry]>,
    _key_spec_storage: Box<[raw::RedisModuleCommandKeySpec]>,
    _args_storage: Box<[Box<[raw::RedisModuleCommandArg]>]>,
    phantom: PhantomData<&'a ()>,
}

impl RawRedisModuleCommandInfo<'_> {
    fn ffi_mut(&mut self) -> &mut raw::RedisModuleCommandInfo {
        &mut self.ffi
    }
}

impl RedisModuleCommandInfo {
    fn as_ffi(&self) -> RawRedisModuleCommandInfo<'_> {
        static COMMNAD_INFO_VERSION: raw::RedisModuleCommandInfoVersion = raw::RedisModuleCommandInfoVersion {
            version: 1,
            sizeof_historyentry: std::mem::size_of::<raw::RedisModuleCommandHistoryEntry>(),
            sizeof_keyspec: std::mem::size_of::<raw::RedisModuleCommandKeySpec>(),
            sizeof_arg: std::mem::size_of::<raw::RedisModuleCommandArg>(),
        };

        let mut history_vec = self
            .history
            .iter()
            .map(|h| raw::RedisModuleCommandHistoryEntry {
                changes: opt_cstr_ptr(&h.changes),
                since: opt_cstr_ptr(&h.since),
            })
            .collect::<Vec<_>>();
        if !history_vec.is_empty() {
            history_vec.push(unsafe { std::mem::zeroed() });
        }

        let mut history_storage = history_vec.into_boxed_slice();

        let mut key_spec_vec = self
            .key_specs
            .iter()
            .map(|spec| raw::RedisModuleCommandKeySpec {
                notes: opt_cstr_ptr(&spec.notes),
                flags: spec.flags.iter().fold(0, |mut flags, flag| {
                    flags |= *flag as u64;
                    flags
                }),
                begin_search_type: match spec.begin_search {
                    KeySpecBeginSearch::Index(_) => raw::RedisModuleKeySpecBeginSearchType_REDISMODULE_KSPEC_BS_INDEX,
                    KeySpecBeginSearch::Keyword { .. } => {
                        raw::RedisModuleKeySpecBeginSearchType_REDISMODULE_KSPEC_BS_KEYWORD
                    }
                },
                bs: match &spec.begin_search {
                    KeySpecBeginSearch::Index(idx) => raw::RedisModuleCommandKeySpec__bindgen_ty_1 {
                        index: raw::RedisModuleCommandKeySpec__bindgen_ty_1__bindgen_ty_1 { pos: *idx },
                    },
                    KeySpecBeginSearch::Keyword { keyword, start_from } => raw::RedisModuleCommandKeySpec__bindgen_ty_1 {
                        keyword: raw::RedisModuleCommandKeySpec__bindgen_ty_1__bindgen_ty_2 {
                            keyword: keyword.as_ptr(),
                            startfrom: *start_from,
                        },
                    },
                },
                find_keys_type: match spec.find_keys {
                    Some(KeySpecFindKeys::Keynum { .. }) => raw::RedisModuleKeySpecFindKeysType_REDISMODULE_KSPEC_FK_KEYNUM,
                    Some(KeySpecFindKeys::Range { .. }) => raw::RedisModuleKeySpecFindKeysType_REDISMODULE_KSPEC_FK_RANGE,
                    None => raw::RedisModuleKeySpecFindKeysType_REDISMODULE_KSPEC_FK_OMITTED,
                },
                fk: match spec.find_keys {
                    Some(KeySpecFindKeys::Keynum {
                        first_key,
                        key_step,
                        keynum_idx,
                    }) => raw::RedisModuleCommandKeySpec__bindgen_ty_2 {
                        keynum: raw::RedisModuleCommandKeySpec__bindgen_ty_2__bindgen_ty_2 {
                            firstkey: first_key,
                            keynumidx: keynum_idx,
                            keystep: key_step,
                        },
                    },
                    Some(KeySpecFindKeys::Range {
                        key_step,
                        last_key,
                        limit,
                    }) => raw::RedisModuleCommandKeySpec__bindgen_ty_2 {
                        range: raw::RedisModuleCommandKeySpec__bindgen_ty_2__bindgen_ty_1 {
                            keystep: key_step,
                            lastkey: last_key,
                            limit: limit,
                        },
                    },
                    None => unsafe { std::mem::zeroed() },
                },
            })
            .collect::<Vec<_>>();
        if !key_spec_vec.is_empty() {
            key_spec_vec.push(unsafe { std::mem::zeroed() });
        }

        let mut key_spec_storage = key_spec_vec.into_boxed_slice();

        fn convert(
            arg_vec: &mut Vec<Box<[raw::RedisModuleCommandArg]>>,
            arg: &RedisModuleCommandArg,
        ) -> raw::RedisModuleCommandArg {
            let mut sub_args = arg.sub_args.iter().map(|arg| convert(arg_vec, arg)).collect::<Vec<_>>();
            let subargs = if !sub_args.is_empty() {
                sub_args.push(unsafe { std::mem::zeroed() });
                let mut sub_args = sub_args.into_boxed_slice();
                let ptr = sub_args.as_mut_ptr();
                arg_vec.push(sub_args);
                ptr
            } else {
                std::ptr::null_mut()
            };
            raw::RedisModuleCommandArg {
                name: arg.name.as_ptr(),
                type_: arg.kind as u32,
                key_spec_index: arg.key_spec_idx,
                token: opt_cstr_ptr(&arg.token),
                summary: opt_cstr_ptr(&arg.summary),
                since: opt_cstr_ptr(&arg.since),
                flags: arg.flags,
                deprecated_since: opt_cstr_ptr(&arg.deprecated_since),
                subargs,
                display_text: opt_cstr_ptr(&arg.display_text),
            }
        }

        let mut args_storage_vec = Vec::new();
        let mut args_vec = self
            .args
            .iter()
            .map(|arg| convert(&mut args_storage_vec, arg))
            .collect::<Vec<_>>();
        let args = if !args_vec.is_empty() {
            args_vec.push(unsafe { std::mem::zeroed() });
            let mut args_vec = args_vec.into_boxed_slice();
            let ptr = args_vec.as_mut_ptr();
            args_storage_vec.push(args_vec);
            ptr
        } else {
            std::ptr::null_mut()
        };

        RawRedisModuleCommandInfo {
            ffi: raw::RedisModuleCommandInfo {
                version: &raw const COMMNAD_INFO_VERSION,
                summary: opt_cstr_ptr(&self.summary),
                complexity: opt_cstr_ptr(&self.complexity),
                since: opt_cstr_ptr(&self.since),
                history: if history_storage.is_empty() {
                    std::ptr::null_mut()
                } else {
                    history_storage.as_mut_ptr()
                },
                tips: opt_cstr_ptr(&self.tips),
                arity: self.arity,
                key_specs: if key_spec_storage.is_empty() {
                    std::ptr::null_mut()
                } else {
                    key_spec_storage.as_mut_ptr()
                },
                args,
            },
            _history_storage: history_storage,
            _args_storage: args_storage_vec.into_boxed_slice(),
            _key_spec_storage: key_spec_storage,
            phantom: PhantomData,
        }
    }
}
