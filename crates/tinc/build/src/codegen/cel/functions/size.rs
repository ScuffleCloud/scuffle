use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub struct Size;

impl Function for Size {
    fn name(&self) -> &'static str {
        "size"
    }

    fn syntax(&self) -> &'static str {
        "<this>.size()"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = ctx.this else {
            return Err(CompileError::syntax("missing this", self));
        };

        if !ctx.args.is_empty() {
            return Err(CompileError::syntax("takes no arguments", self));
        }

        if let CompiledExpr::Runtime(RuntimeCompiledExpr {
            expr,
            ty: CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(_) | ProtoModifiedValueType::Map(_, _))),
        }) = &this
        {
            return Ok(CompiledExpr::runtime(
                CelType::Proto(ProtoType::Value(ProtoValueType::UInt64)),
                parse_quote! {
                    ((#expr).len() as u64)
                },
            ));
        }

        match this.to_cel()? {
            CompiledExpr::Constant(ConstantCompiledExpr { value }) => Ok(CompiledExpr::constant(CelValue::cel_size(value)?)),
            CompiledExpr::Runtime(RuntimeCompiledExpr { expr, .. }) => Ok(CompiledExpr::runtime(
                CelType::Proto(ProtoType::Value(ProtoValueType::UInt64)),
                parse_quote!(::tinc::__private::cel::CelValue::cel_size(#expr)?),
            )),
        }
    }
}
