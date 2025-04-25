use cel_interpreter::{ExecutionError, FunctionContext};

mod contains;
mod size;

pub use contains::Contains;
pub use size::Size;

use super::compiler::{CompileError, CompiledExpr, Compiler, CompilerCtx};

pub trait Function: Sized + 'static {
    const NAME: &'static str;

    fn compile(ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let _ = ctx;
        Err(CompileError::NotImplemented)
    }

    fn interpret(fctx: &FunctionContext) -> Result<cel_interpreter::Value, ExecutionError> {
        let _ = fctx;
        Err(ExecutionError::not_supported_as_method(
            Self::NAME,
            cel_interpreter::Value::Null,
        ))
    }

    fn add_to_ctx(ctx: &mut cel_interpreter::Context) {
        ctx.add_function(Self::NAME, Self::interpret);
    }

    fn add_to_compiler(ctx: &mut Compiler) {
        ctx.register_function::<Self>();
    }
}
