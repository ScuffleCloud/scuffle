use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};

#[derive(Debug, Clone, Default)]
pub(crate) struct Bool;

impl Function for Bool {
    fn name(&self) -> &'static str {
        "bool"
    }

    fn syntax(&self) -> &'static str {
        "<this>.bool()"
    }

    fn compile(&self, mut ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = ctx.this.take() else {
            return Err(CompileError::syntax("missing this", self));
        };

        if !ctx.args.is_empty() {
            return Err(CompileError::syntax("takes no arguments", self));
        }

        Ok(this.into_bool(&ctx))
    }
}
