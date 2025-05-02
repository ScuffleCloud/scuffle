use syn::parse_quote;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub struct Size;

impl Function for Size {
    fn name(&self) -> &'static str {
        "size"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = &ctx.this else {
            return Err(CompileError::MissingTarget {
                func: self.name(),
                message: "this is required when calling the size function".to_string(),
            });
        };

        if !ctx.args.is_empty() {
            return Err(CompileError::InvalidFunctionArgumentCount {
                func: self.name(),
                expected: 0,
                got: ctx.args.len(),
            });
        }

        match &this.ty {
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(_, _) | ProtoModifiedValueType::Repeated(_))) => {
                Ok(CompiledExpr {
                    expr: parse_quote! {
                        (#this).len() as u64
                    },
                    ty: CelType::Proto(ProtoType::Value(ProtoValueType::UInt64)),
                })
            }
            _ => {
                let this = this.clone().to_cel()?;
                Ok(CompiledExpr {
                    expr: parse_quote! {
                        ::tinc::__private::cel::CelValue::cel_size(#this)?
                    },
                    ty: CelType::Proto(ProtoType::Value(ProtoValueType::UInt64)),
                })
            }
        }
    }

    fn interpret(
        &self,
        fctx: &cel_interpreter::FunctionContext,
    ) -> Result<cel_interpreter::Value, cel_interpreter::ExecutionError> {
        let Some(this) = &fctx.this else {
            return Err(cel_interpreter::ExecutionError::missing_argument_or_target());
        };

        if !fctx.args.is_empty() {
            return Err(cel_interpreter::ExecutionError::invalid_argument_count(0, fctx.args.len()));
        }

        match this {
            cel_interpreter::Value::String(s) => Ok(cel_interpreter::Value::UInt(s.len() as u64)),
            cel_interpreter::Value::Bytes(b) => Ok(cel_interpreter::Value::UInt(b.len() as u64)),
            cel_interpreter::Value::List(l) => Ok(cel_interpreter::Value::UInt(l.len() as u64)),
            cel_interpreter::Value::Map(m) => Ok(cel_interpreter::Value::UInt(m.map.len() as u64)),
            v => Err(cel_interpreter::ExecutionError::unsupported_target_type(v.clone())),
        }
    }
}
