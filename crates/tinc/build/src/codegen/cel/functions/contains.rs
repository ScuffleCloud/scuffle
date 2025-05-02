use cel_interpreter::{ExecutionError, FunctionContext};
use quote::quote;
use syn::parse_quote;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub struct Contains;

// this.contains(arg) -> arg in this
impl Function for Contains {
    fn name(&self) -> &'static str {
        "contains"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = &ctx.this else {
            return Err(CompileError::MissingTarget {
                func: self.name(),
                message: "this is required when calling the contains function".to_string(),
            });
        };

        if ctx.args.len() != 1 {
            return Err(CompileError::InvalidFunctionArgumentCount {
                func: self.name(),
                expected: 1,
                got: ctx.args.len(),
            });
        }

        let arg = ctx.resolve(&ctx.args[0])?.to_cel()?;

        if let CelType::Proto(ProtoType::Modified(
            ProtoModifiedValueType::Repeated(item) | ProtoModifiedValueType::Map(item, _),
        )) = &this.ty
        {
            if !matches!(item, ProtoValueType::Message { .. } | ProtoValueType::Enum(_)) {
                let op = match &this.ty {
                    CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(_))) => {
                        quote! { array_contains }
                    }
                    CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(_, _))) => {
                        quote! { map_contains }
                    }
                    _ => unreachable!(),
                };

                return Ok(CompiledExpr {
                    expr: parse_quote! {
                        ::tinc::__private::cel::#op(
                            #this,
                            #arg,
                        )
                    },
                    ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                });
            }
        }

        let this = this.clone().to_cel()?;

        Ok(CompiledExpr {
            expr: parse_quote! {
                ::tinc::__private::cel::CelValue::cel_contains(
                    #this,
                    #arg,
                )?
            },
            ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
        })
    }

    fn interpret(&self, fctx: &FunctionContext) -> Result<cel_interpreter::Value, ExecutionError> {
        let Some(this) = &fctx.this else {
            return Err(ExecutionError::missing_argument_or_target());
        };

        if fctx.args.len() != 1 {
            return Err(ExecutionError::invalid_argument_count(1, fctx.args.len()));
        }

        let arg = fctx.ptx.resolve(&fctx.args[0])?;

        match (this, arg) {
            (cel_interpreter::Value::String(s), cel_interpreter::Value::String(t)) => {
                Ok(cel_interpreter::Value::Bool(s.contains(t.as_str())))
            }
            (cel_interpreter::Value::Bytes(s), cel_interpreter::Value::Bytes(t)) => {
                Ok(cel_interpreter::Value::Bool(s.windows(t.len()).any(|w| w == t.as_slice())))
            }
            (cel_interpreter::Value::List(s), value) => Ok(cel_interpreter::Value::Bool(s.contains(&value))),
            (cel_interpreter::Value::Map(s), value) => {
                let key: Option<cel_interpreter::objects::Key> = value.clone().try_into().ok();
                Ok(cel_interpreter::Value::Bool(key.is_some_and(|k| s.get(&k).is_some())))
            }
            _ => Ok(cel_interpreter::Value::Bool(false)),
        }
    }
}
