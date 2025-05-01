use syn::parse_quote;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoType, ProtoValueType};

pub struct Bool;

impl Function for Bool {
    const NAME: &'static str = "bool";

    fn compile(ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        if ctx.this.is_some() {
            return Err(CompileError::MissingTarget {
                func: Self::NAME,
                message: "bad usage for bool(arg) function".to_string(),
            });
        }

        if ctx.args.len() != 1 {
            return Err(CompileError::InvalidFunctionArgumentCount {
                func: Self::NAME,
                expected: 1,
                got: ctx.args.len(),
            });
        }

        let arg = ctx.resolve(&ctx.args[0])?;

        if !arg.ty.can_be_cel() {
            return Err(CompileError::TypeConversion {
                ty: arg.ty.into(),
                message: "The return type must be a CEL compatiable type".into(),
            });
        }

        Ok(CompiledExpr {
            expr: parse_quote! {
                ::tinc::__private::cel::to_bool(#arg)?
            },
            ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
        })
    }

    fn interpret(
        fctx: &cel_interpreter::FunctionContext,
    ) -> Result<cel_interpreter::Value, cel_interpreter::ExecutionError> {
        if fctx.this.is_some() {
            return Err(cel_interpreter::ExecutionError::missing_argument_or_target());
        };

        if fctx.args.len() != 1 {
            return Err(cel_interpreter::ExecutionError::invalid_argument_count(1, fctx.args.len()));
        }

        let value = fctx.ptx.resolve(&fctx.args[0])?;

        Ok(match value {
            cel_interpreter::Value::Int(i) => cel_interpreter::Value::Bool(i != 0),
            cel_interpreter::Value::Float(i) => cel_interpreter::Value::Bool(i != 0.0),
            cel_interpreter::Value::UInt(i) => cel_interpreter::Value::Bool(i != 0),
            cel_interpreter::Value::String(s) => cel_interpreter::Value::Bool(!s.is_empty()),
            cel_interpreter::Value::Bool(b) => cel_interpreter::Value::Bool(b),
            cel_interpreter::Value::Bytes(b) => cel_interpreter::Value::Bool(!b.is_empty()),
            cel_interpreter::Value::Duration(d) => cel_interpreter::Value::Bool(!d.is_zero()),
            cel_interpreter::Value::Timestamp(t) => {
                cel_interpreter::Value::Bool(t.timestamp_nanos_opt().is_some_and(|v| v != 0))
            }
            cel_interpreter::Value::List(l) => cel_interpreter::Value::Bool(!l.is_empty()),
            cel_interpreter::Value::Function(_, _) => cel_interpreter::Value::Bool(false),
            cel_interpreter::Value::Map(m) => cel_interpreter::Value::Bool(!m.map.is_empty()),
            cel_interpreter::Value::Null => cel_interpreter::Value::Bool(false),
        })
    }
}
