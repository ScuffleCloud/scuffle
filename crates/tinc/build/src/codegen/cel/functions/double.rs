use num_traits::cast::ToPrimitive;
use syn::parse_quote;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};
use crate::codegen::cel::types::CelType;

#[derive(Debug, Clone, Default)]
pub struct Double;

impl Function for Double {
    fn name(&self) -> &'static str {
        "double"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        if ctx.this.is_some() {
            return Err(CompileError::MissingTarget {
                func: self.name(),
                message: "bad usage for double(arg) function".to_string(),
            });
        }

        if ctx.args.len() != 1 {
            return Err(CompileError::InvalidFunctionArgumentCount {
                func: self.name(),
                expected: 1,
                got: ctx.args.len(),
            });
        }

        let arg = ctx.resolve(&ctx.args[0])?.to_cel()?;

        Ok(CompiledExpr {
            expr: parse_quote! {
                ::tinc::__private::cel::CelValue::cel_to_double(#arg)?
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

        Ok(match value {
            cel_interpreter::Value::Int(i) => match i.to_f64() {
                Some(i) => cel_interpreter::Value::Float(i),
                None => cel_interpreter::Value::Null,
            },
            cel_interpreter::Value::UInt(i) => match i.to_f64() {
                Some(i) => cel_interpreter::Value::Float(i),
                None => cel_interpreter::Value::Null,
            },
            cel_interpreter::Value::Float(i) => cel_interpreter::Value::Float(i),
            cel_interpreter::Value::String(s) => {
                if let Ok(i) = s.parse() {
                    cel_interpreter::Value::Float(i)
                } else {
                    cel_interpreter::Value::Null
                }
            }
            cel_interpreter::Value::Bool(b) => cel_interpreter::Value::Float(if b { 1.0 } else { 0.0 }),
            target => return Err(cel_interpreter::ExecutionError::UnsupportedTargetType { target }),
        })
    }
}
