use syn::parse_quote;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};
use crate::codegen::cel::types::CelType;
use crate::codegen::cel::value_to_str;

pub struct String;

impl Function for String {
    const NAME: &'static str = "string";

    fn compile(ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        if ctx.this.is_some() {
            return Err(CompileError::MissingTarget {
                func: Self::NAME,
                message: format!("bad usage for string(arg) function"),
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
                ::tinc::__private::cel::CelValue::cel_to_string(#arg)?
            },
            ty: CelType::CelValue,
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

        Ok(cel_interpreter::Value::String(value_to_str(&value).to_string().into()))
    }
}
