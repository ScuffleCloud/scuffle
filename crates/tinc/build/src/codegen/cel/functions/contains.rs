use quote::quote;
use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub struct Contains;

// this.contains(arg)
// arg in this
impl Function for Contains {
    fn name(&self) -> &'static str {
        "contains"
    }

    fn syntax(&self) -> &'static str {
        "<this>.contains(<arg>)"
    }

    fn compile(&self, mut ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = ctx.this.take() else {
            return Err(CompileError::syntax("missing this", self));
        };

        if ctx.args.len() != 1 {
            return Err(CompileError::syntax("takes exactly one argument", self));
        }

        let arg = ctx.resolve(&ctx.args[0])?.to_cel()?;

        if let CompiledExpr::Runtime(RuntimeCompiledExpr {
            expr,
            ty:
                ty @ CelType::Proto(ProtoType::Modified(
                    ProtoModifiedValueType::Repeated(item) | ProtoModifiedValueType::Map(item, _),
                )),
        }) = &this
        {
            if !matches!(item, ProtoValueType::Message { .. } | ProtoValueType::Enum(_)) {
                let op = match &ty {
                    CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(_))) => {
                        quote! { array_contains }
                    }
                    CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(_, _))) => {
                        quote! { map_contains }
                    }
                    _ => unreachable!(),
                };

                return Ok(CompiledExpr::runtime(
                    CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                    parse_quote! {
                        ::tinc::__private::cel::#op(
                            #expr,
                            #arg,
                        )
                    },
                ));
            }
        }

        let this = this.clone().to_cel()?;

        match (this, arg) {
            (
                CompiledExpr::Constant(ConstantCompiledExpr { value: this }),
                CompiledExpr::Constant(ConstantCompiledExpr { value: arg }),
            ) => Ok(CompiledExpr::constant(CelValue::cel_contains(this, arg)?)),
            (this, arg) => Ok(CompiledExpr::runtime(
                CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                parse_quote! {
                    ::tinc::__private::cel::CelValue::cel_contains(
                        #this,
                        #arg,
                    )?
                },
            )),
        }
    }
}
