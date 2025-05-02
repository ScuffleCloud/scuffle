use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub struct StartsWith;

// this.stratsWith(arg) -> arg in this
impl Function for StartsWith {
    fn name(&self) -> &'static str {
        "startsWith"
    }

    fn syntax(&self) -> &'static str {
        "<this>.startsWith(<arg>)"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = &ctx.this else {
            return Err(CompileError::syntax("missing this", self));
        };

        if ctx.args.len() != 1 {
            return Err(CompileError::syntax("takes exactly one argument", self));
        }

        let arg = ctx.resolve(&ctx.args[0])?.to_cel()?;
        let this = this.clone().to_cel()?;

        match (this, arg) {
            (
                CompiledExpr::Constant(ConstantCompiledExpr { value: this }),
                CompiledExpr::Constant(ConstantCompiledExpr { value: arg }),
            ) => Ok(CompiledExpr::constant(CelValue::cel_starts_with(this, arg)?)),
            (this, arg) => Ok(CompiledExpr::runtime(
                CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                parse_quote! {
                    ::tinc::__private::cel::CelValue::cel_starts_with(
                        #this,
                        #arg,
                    )?
                },
            )),
        }
    }
}
