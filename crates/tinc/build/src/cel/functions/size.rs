use syn::parse_quote;

use super::Function;
use crate::cel::codegen::{CelType, ProtoModifiedValueType, ProtoType, ProtoValueType};
use crate::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};

pub struct Size;

impl Function for Size {
    const NAME: &'static str = "size";

    fn compile(ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = &ctx.this else {
            return Err(CompileError::MissingTarget {
                func: Self::NAME,
                message: "this is required when calling the size function".to_string(),
            });
        };

        if !ctx.args.is_empty() {
            return Err(CompileError::InvalidFunctionArgumentCount {
                func: Self::NAME,
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
            _ => Ok(CompiledExpr {
                expr: parse_quote! {
                    ::tinc::__private::cel::CelValue::size(#this)?
                },
                ty: CelType::Proto(ProtoType::Value(ProtoValueType::UInt64)),
            }),
        }
    }

    fn interpret(
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
