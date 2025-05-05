mod all;
mod bool;
mod bytes;
mod contains;
mod double;
mod dyn_;
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

pub(crate) use all::All;
pub(crate) use bool::Bool;
pub(crate) use bytes::Bytes;
pub(crate) use contains::Contains;
pub(crate) use double::Double;
pub(crate) use dyn_::Dyn;
pub(crate) use ends_with::EndsWith;
pub(crate) use enum_::Enum;
pub(crate) use exists::Exists;
pub(crate) use exists_one::ExistsOne;
pub(crate) use filter::Filter;
pub(crate) use has::Has;
pub(crate) use int::Int;
pub(crate) use is_email::IsEmail;
pub(crate) use is_hostname::IsHostname;
pub(crate) use is_ipv4::IsIpv4;
pub(crate) use is_ipv6::IsIpv6;
pub(crate) use is_uri::IsUri;
pub(crate) use is_uuid::IsUuid;
pub(crate) use map::Map;
pub(crate) use matches::Matches;
pub(crate) use size::Size;
pub(crate) use starts_with::StartsWith;
pub(crate) use string::String;
pub(crate) use uint::UInt;

use super::compiler::{CompileError, CompiledExpr, Compiler, CompilerCtx};

pub(crate) fn add_to_compiler(compiler: &mut Compiler) {
    Contains.add_to_compiler(compiler);
    Size.add_to_compiler(compiler);
    Has.add_to_compiler(compiler);
    Map.add_to_compiler(compiler);
    Filter.add_to_compiler(compiler);
    All.add_to_compiler(compiler);
    Exists.add_to_compiler(compiler);
    ExistsOne.add_to_compiler(compiler);
    StartsWith.add_to_compiler(compiler);
    EndsWith.add_to_compiler(compiler);
    Matches.add_to_compiler(compiler);
    String.add_to_compiler(compiler);
    Bytes.add_to_compiler(compiler);
    Int.add_to_compiler(compiler);
    UInt.add_to_compiler(compiler);
    Double.add_to_compiler(compiler);
    Bool.add_to_compiler(compiler);
    Enum::default().add_to_compiler(compiler);
    IsIpv4.add_to_compiler(compiler);
    IsIpv6.add_to_compiler(compiler);
    IsUuid.add_to_compiler(compiler);
    IsHostname.add_to_compiler(compiler);
    IsUri.add_to_compiler(compiler);
    IsEmail.add_to_compiler(compiler);
    Dyn.add_to_compiler(compiler);
}

pub(crate) trait Function: Send + Sync + 'static {
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
