use syn::parse_quote;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoPath, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub struct Enum(pub Option<ProtoPath>);

impl Function for Enum {
    fn name(&self) -> &'static str {
        "enum"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = ctx.this.as_ref() else {
            return Err(CompileError::MissingTarget {
                func: self.name(),
                message: "enum must have a target".to_owned(),
            });
        };

        if ctx.args.len() > 1 {
            return Err(CompileError::InvalidFunctionArgumentCount {
                func: self.name(),
                expected: 1,
                got: ctx.args.len(),
            });
        }

        let enum_path = if let Some(arg) = ctx.args.get(0) {
            ctx.resolve(arg)?
        } else {
            match (&this.ty, &self.0) {
                (
                    CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Enum(path))))
                    | CelType::Proto(ProtoType::Value(ProtoValueType::Enum(path))),
                    _,
                )
                | (_, Some(path)) => {
                    let path = path.as_ref();
                    CompiledExpr {
                        expr: parse_quote! { #path },
                        ty: CelType::Proto(ProtoType::Value(ProtoValueType::String)),
                    }
                }
                _ => {
                    return Err(CompileError::MissingTarget {
                        func: self.name(),
                        message: "Unable to determine what enum to convert to, try providing a path in the first argument"
                            .to_string(),
                    });
                }
            }
        };

        let this = this.clone().to_cel()?;
        let enum_path = enum_path.to_cel()?;

        Ok(CompiledExpr {
            expr: parse_quote! {
                ::tinc::__private::cel::CelValue::cel_to_enum(
                    #this,
                    #enum_path,
                )
            },
            ty: CelType::CelValue,
        })
    }

    fn interpret(
        &self,
        _: &cel_interpreter::FunctionContext,
    ) -> Result<cel_interpreter::Value, cel_interpreter::ExecutionError> {
        Err(cel_interpreter::ExecutionError::FunctionError {
            function: self.name().to_owned(),
            message: "enum function must be evaluated at runtime".to_owned(),
        })
    }
}
