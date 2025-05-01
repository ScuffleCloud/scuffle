use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};

pub struct Dyn;

impl Function for Dyn {
    const NAME: &'static str = "dyn";

    fn compile(ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        if ctx.this.is_some() {
            return Err(CompileError::MissingTarget {
                func: Self::NAME,
                message: "dyn cannot have a target".to_owned(),
            });
        }

        if ctx.args.len() != 1 {
            return Err(CompileError::InvalidFunctionArgumentCount {
                func: Self::NAME,
                expected: 1,
                got: ctx.args.len(),
            });
        }

        ctx.resolve(&ctx.args[0])
    }

    fn interpret(_: &cel_interpreter::FunctionContext) -> Result<cel_interpreter::Value, cel_interpreter::ExecutionError> {
        Err(cel_interpreter::ExecutionError::FunctionError {
            function: Self::NAME.to_owned(),
            message: "dyn function must be evaluated at runtime".to_owned(),
        })
    }
}
