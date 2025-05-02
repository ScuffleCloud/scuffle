use syn::parse_quote;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};
use crate::codegen::cel::types::CelType;

#[derive(Debug, Clone, Default)]
pub struct Bytes;

impl Function for Bytes {
    fn name(&self) -> &'static str {
        "bytes"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        if ctx.this.is_some() {
            return Err(CompileError::MissingTarget {
                func: self.name(),
                message: "bad usage for bytes(arg) function".to_string(),
            });
        }

        if ctx.args.len() != 1 {
            return Err(CompileError::InvalidFunctionArgumentCount {
                func: self.name(),
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
                ::tinc::__private::cel::CelValue::cel_to_bytes(#arg)?
            },
            ty: CelType::CelValue,
        })
    }

    fn interpret(
        &self,
        fctx: &cel_interpreter::FunctionContext,
    ) -> Result<cel_interpreter::Value, cel_interpreter::ExecutionError> {
        if fctx.this.is_some() {
            return Err(cel_interpreter::ExecutionError::missing_argument_or_target());
        };

        if fctx.args.len() != 1 {
            return Err(cel_interpreter::ExecutionError::invalid_argument_count(1, fctx.args.len()));
        }

        let value = fctx.ptx.resolve(&fctx.args[0])?;

        Ok(cel_interpreter::Value::Bytes(match value {
            cel_interpreter::Value::Bytes(b) => b,
            cel_interpreter::Value::String(s) => s.as_bytes().to_vec().into(),
            v => return Err(v.error_expected_type(cel_interpreter::objects::ValueType::String)),
        }))
    }
}
