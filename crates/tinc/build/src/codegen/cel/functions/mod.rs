use std::sync::Arc;

use cel_interpreter::{ExecutionError, FunctionContext};

mod all;
mod bool;
mod bytes;
mod const_;
mod contains;
mod double;
mod dyn_;
mod ends_with;
mod exists;
mod exists_one;
mod filter;
mod has;
mod int;
mod map;
mod matches;
mod size;
mod starts_with;
mod string;
mod uint;

pub use all::All;
pub use bool::Bool;
pub use bytes::Bytes;
pub use const_::Const;
pub use contains::Contains;
pub use double::Double;
pub use dyn_::Dyn;
pub use ends_with::EndsWith;
pub use exists::Exists;
pub use exists_one::ExistsOne;
pub use filter::Filter;
pub use has::Has;
pub use int::Int;
pub use map::Map;
pub use matches::Matches;
pub use size::Size;
pub use starts_with::StartsWith;
pub use string::String;
pub use uint::UInt;

use super::compiler::{CompileError, CompiledExpr, Compiler, CompilerCtx};

pub fn add_to_context(ctx: &mut cel_interpreter::Context) {
    Contains.add_to_ctx(ctx);
    Size.add_to_ctx(ctx);
    Has.add_to_ctx(ctx);
    Map.add_to_ctx(ctx);
    Filter.add_to_ctx(ctx);
    All.add_to_ctx(ctx);
    Exists.add_to_ctx(ctx);
    ExistsOne.add_to_ctx(ctx);
    StartsWith.add_to_ctx(ctx);
    EndsWith.add_to_ctx(ctx);
    Matches.add_to_ctx(ctx);
    String.add_to_ctx(ctx);
    Bytes.add_to_ctx(ctx);
    Int.add_to_ctx(ctx);
    UInt.add_to_ctx(ctx);
    Double.add_to_ctx(ctx);
    Bool.add_to_ctx(ctx);
    Const.add_to_ctx(ctx);
    Dyn.add_to_ctx(ctx);
}

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
    compiler.register_function(Const);
    compiler.register_function(Dyn);
}

pub trait Function: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let _ = ctx;
        Err(CompileError::NotImplemented)
    }

    fn interpret(&self, fctx: &FunctionContext) -> Result<cel_interpreter::Value, ExecutionError> {
        let _ = fctx;
        Err(ExecutionError::not_supported_as_method(
            self.name(),
            cel_interpreter::Value::Null,
        ))
    }

    fn add_to_ctx(self, ctx: &mut cel_interpreter::Context)
    where
        Self: Sized,
    {
        let this = Arc::new(self);
        ctx.add_function(this.name(), move |ctx: &FunctionContext| this.interpret(ctx));
    }

    fn add_to_compiler(self, ctx: &mut Compiler)
    where
        Self: Sized,
    {
        ctx.register_function(self);
    }
}
