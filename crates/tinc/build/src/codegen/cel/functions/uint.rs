use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;

#[derive(Debug, Clone, Default)]
pub struct UInt;

impl Function for UInt {
    fn name(&self) -> &'static str {
        "uint"
    }

    fn syntax(&self) -> &'static str {
        "<this>.uint()"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = ctx.this else {
            return Err(CompileError::syntax("missing this", self));
        };

        if !ctx.args.is_empty() {
            return Err(CompileError::syntax("takes no arguments", self));
        }

        match this.to_cel()? {
            CompiledExpr::Constant(ConstantCompiledExpr { value }) => Ok(CompiledExpr::Constant(ConstantCompiledExpr {
                value: CelValue::cel_to_uint(value)?,
            })),
            CompiledExpr::Runtime(RuntimeCompiledExpr { expr, .. }) => Ok(CompiledExpr::Runtime(RuntimeCompiledExpr {
                ty: CelType::CelValue,
                expr: parse_quote!(::tinc::__private::cel::CelValue::cel_to_uint(#expr)?),
            })),
        }
    }
}
