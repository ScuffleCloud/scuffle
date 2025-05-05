use syn::parse_quote;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};
use crate::codegen::cel::types::CelType;

#[derive(Debug, Clone, Default)]
pub(crate) struct Dyn;

impl Function for Dyn {
    fn name(&self) -> &'static str {
        "dyn"
    }

    fn syntax(&self) -> &'static str {
        "dyn(<expr>)"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        if ctx.this.is_some() {
            return Err(CompileError::syntax("has this", self));
        };

        if ctx.args.len() != 1 {
            return Err(CompileError::syntax("needs exactly 1 argument", self));
        }

        let result = ctx.resolve(&ctx.args[0])?;

        let ty = match &result {
            CompiledExpr::Constant(_) => CelType::CelValue,
            CompiledExpr::Runtime(r) => r.ty.clone(),
        };

        Ok(CompiledExpr::runtime(ty, parse_quote!(#result)))
    }
}
