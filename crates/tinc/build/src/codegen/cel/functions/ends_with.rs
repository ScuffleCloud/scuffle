use cel_interpreter::{ExecutionError, FunctionContext};
use syn::parse_quote;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub struct EndsWith;

// this.stratsWith(arg) -> arg in this
impl Function for EndsWith {
    fn name(&self) -> &'static str {
        "endsWith"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = &ctx.this else {
            return Err(CompileError::MissingTarget {
                func: self.name(),
                message: "this is required when calling the endsWith function".to_string(),
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
        let this = this.clone().to_cel()?;

        Ok(CompiledExpr {
            expr: parse_quote! {
                ::tinc::__private::cel::CelValue::cel_ends_with(
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
                Ok(cel_interpreter::Value::Bool(s.ends_with(t.as_str())))
            }
            (cel_interpreter::Value::Bytes(s), cel_interpreter::Value::Bytes(t)) => {
                Ok(cel_interpreter::Value::Bool(s.ends_with(t.as_slice())))
            }
            _ => Ok(cel_interpreter::Value::Bool(false)),
        }
    }
}
