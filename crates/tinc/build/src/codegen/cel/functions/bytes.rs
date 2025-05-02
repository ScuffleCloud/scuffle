use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;

#[derive(Debug, Clone, Default)]
pub struct Bytes;

impl Function for Bytes {
    fn name(&self) -> &'static str {
        "bytes"
    }

    fn syntax(&self) -> &'static str {
        "<this>.bytes()"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = ctx.this else {
            return Err(CompileError::syntax("missing this", self));
        };

        if !ctx.args.is_empty() {
            return Err(CompileError::syntax("takes no arguments", self));
        }

        match this.to_cel()? {
            CompiledExpr::Constant(ConstantCompiledExpr { value }) => {
                Ok(CompiledExpr::constant(CelValue::cel_to_bytes(value)?))
            }
            CompiledExpr::Runtime(RuntimeCompiledExpr { expr, .. }) => Ok(CompiledExpr::runtime(
                CelType::CelValue,
                parse_quote!(::tinc::__private::cel::CelValue::cel_to_bytes(#expr)?),
            )),
        }
    }
}
