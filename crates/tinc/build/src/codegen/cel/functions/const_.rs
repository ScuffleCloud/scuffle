use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};

#[derive(Debug, Clone, Default)]
pub struct Const;

impl Function for Const {
    fn name(&self) -> &'static str {
        "const"
    }

    fn compile(&self, _: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        Err(CompileError::FunctionNotFound(
            "const must be evaluated at compile time".into(),
        ))
    }

    fn interpret(
        &self,
        fctx: &cel_interpreter::FunctionContext,
    ) -> Result<cel_interpreter::Value, cel_interpreter::ExecutionError> {
        if fctx.this.is_some() {
            return Err(cel_interpreter::ExecutionError::missing_argument_or_target());
        };

        if fctx.args.len() != 1 {
            return Err(cel_interpreter::ExecutionError::invalid_argument_count(1, fctx.args.len()));
        }

        fctx.ptx.resolve(&fctx.args[0])
    }
}
