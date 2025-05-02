use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};

#[derive(Debug, Clone, Default)]
pub struct Dyn;

impl Function for Dyn {
    fn name(&self) -> &'static str {
        "dyn"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        if ctx.this.is_some() {
            return Err(CompileError::MissingTarget {
                func: self.name(),
                message: "dyn cannot have a target".to_owned(),
            });
        }

        if ctx.args.len() != 1 {
            return Err(CompileError::InvalidFunctionArgumentCount {
                func: self.name(),
                expected: 1,
                got: ctx.args.len(),
            });
        }

        ctx.resolve(&ctx.args[0])
    }

    fn interpret(
        &self,
        _: &cel_interpreter::FunctionContext,
    ) -> Result<cel_interpreter::Value, cel_interpreter::ExecutionError> {
        Err(cel_interpreter::ExecutionError::FunctionError {
            function: self.name().to_owned(),
            message: "dyn function must be evaluated at runtime".to_owned(),
        })
    }
}
