mod all;
mod bool;
mod bytes;
mod contains;
mod double;
mod ends_with;
mod enum_;
mod exists;
mod exists_one;
mod filter;
mod has;
mod int;
mod is_email;
mod is_hostname;
mod is_ipv4;
mod is_ipv6;
mod is_uri;
mod is_uuid;
mod map;
mod matches;
mod size;
mod starts_with;
mod string;
mod uint;

pub use all::All;
pub use bool::Bool;
pub use bytes::Bytes;
pub use contains::Contains;
pub use double::Double;
pub use ends_with::EndsWith;
pub use enum_::Enum;
pub use exists::Exists;
pub use exists_one::ExistsOne;
pub use filter::Filter;
pub use has::Has;
pub use int::Int;
pub use is_email::IsEmail;
pub use is_hostname::IsHostname;
pub use is_ipv4::IsIpv4;
pub use is_ipv6::IsIpv6;
pub use is_uri::IsUri;
pub use is_uuid::IsUuid;
pub use map::Map;
pub use matches::Matches;
pub use size::Size;
pub use starts_with::StartsWith;
pub use string::String;
pub use uint::UInt;

use super::compiler::{CompileError, CompiledExpr, Compiler, CompilerCtx};

pub fn add_to_compiler(compiler: &mut Compiler) {
    compiler.register_function(Contains);
    compiler.register_function(Size);
    compiler.register_function(Has);
    compiler.register_function(Map);
    compiler.register_function(Filter);
    compiler.register_function(All);
    compiler.register_function(Exists);
    compiler.register_function(ExistsOne);
    compiler.register_function(StartsWith);
    compiler.register_function(EndsWith);
    compiler.register_function(Matches);
    compiler.register_function(String);
    compiler.register_function(Bytes);
    compiler.register_function(Int);
    compiler.register_function(UInt);
    compiler.register_function(Double);
    compiler.register_function(Bool);
    compiler.register_function(Enum::default());
    compiler.register_function(IsIpv4);
    compiler.register_function(IsIpv6);
    compiler.register_function(IsUuid);
    compiler.register_function(IsHostname);
    compiler.register_function(IsUri);
    compiler.register_function(IsEmail);
}

pub trait Function: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    fn syntax(&self) -> &'static str;

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let _ = ctx;
        Err(CompileError::NotImplemented)
    }

    fn add_to_compiler(self, ctx: &mut Compiler)
    where
        Self: Sized,
    {
        ctx.register_function(self);
    }
}
