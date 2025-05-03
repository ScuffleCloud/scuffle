use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoPath, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub(crate) struct Enum(pub Option<ProtoPath>);

impl Function for Enum {
    fn name(&self) -> &'static str {
        "enum"
    }

    fn syntax(&self) -> &'static str {
        "<this>.enum() | <this>.enum(<path>)"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = ctx.this.as_ref() else {
            return Err(CompileError::syntax("missing this", self));
        };

        if ctx.args.len() > 1 {
            return Err(CompileError::syntax("invalid number of arguments", self));
        }

        let enum_path = if let Some(arg) = ctx.args.first() {
            ctx.resolve(arg)?
        } else {
            match (&this, &self.0) {
                (
                    CompiledExpr::Runtime(RuntimeCompiledExpr {
                        ty:
                            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Enum(path))))
                            | CelType::Proto(ProtoType::Value(ProtoValueType::Enum(path))),
                        ..
                    }),
                    _,
                )
                | (_, Some(path)) => CompiledExpr::Constant(ConstantCompiledExpr {
                    value: CelValue::String(path.0.clone().into()),
                }),
                _ => {
                    return Err(CompileError::syntax(
                        "unable to determine enum type, try providing an explicit path",
                        self,
                    ));
                }
            }
        };

        let this = this.clone().into_cel()?;
        let enum_path = enum_path.into_cel()?;

        match (this, enum_path) {
            (
                CompiledExpr::Constant(ConstantCompiledExpr { value: this }),
                CompiledExpr::Constant(ConstantCompiledExpr { value: enum_path }),
            ) => Ok(CompiledExpr::constant(CelValue::cel_to_enum(this, enum_path)?)),
            (this, enum_path) => Ok(CompiledExpr::runtime(
                CelType::CelValue,
                parse_quote! {
                    ::tinc::__private::cel::CelValue::cel_to_enum(
                        #this,
                        #enum_path,
                    )?
                },
            )),
        }
    }
}
