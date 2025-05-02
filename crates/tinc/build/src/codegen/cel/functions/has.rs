use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};

#[derive(Debug, Clone, Default)]
pub struct Has;

// has(field-arg)
impl Function for Has {
    fn name(&self) -> &'static str {
        "has"
    }

    fn syntax(&self) -> &'static str {
        "has(<field accessor>)"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        if ctx.this.is_some() {
            return Err(CompileError::syntax("function has no this", self));
        };

        if ctx.args.len() != 1 {
            return Err(CompileError::syntax("invalid arguments", self));
        }

        let arg = ctx.resolve(&ctx.args[0]);

        Ok(CompiledExpr::constant(arg.is_ok()))
    }
}
