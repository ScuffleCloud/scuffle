use cel_interpreter::{ExecutionError, FunctionContext};
use syn::parse_quote;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoType, ProtoValueType};

pub struct Has;

// has(field-arg)
impl Function for Has {
    const NAME: &'static str = "has";

    fn compile(ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        if ctx.this.is_some() {
            return Err(CompileError::MissingTarget {
                func: Self::NAME,
                message: "this function does not have a target".to_string(),
            });
        };

        if ctx.args.len() != 1 {
            return Err(CompileError::InvalidFunctionArgumentCount {
                func: Self::NAME,
                expected: 1,
                got: ctx.args.len(),
            });
        }

        let arg = ctx.resolve(&ctx.args[0]);

        Ok(CompiledExpr {
            expr: match arg {
                Ok(arg) => parse_quote! {
                    (|| {
                        #arg
                        ::core::result::Result::Ok::<(), ::tinc::__private::cel::CelError>(())
                    }).is_ok()
                },
                Err(_) => parse_quote! {
                    false
                },
            },
            ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
        })
    }

    fn interpret(fctx: &FunctionContext) -> Result<cel_interpreter::Value, ExecutionError> {
        if fctx.this.is_some() {
            return Err(ExecutionError::missing_argument_or_target());
        }

        if fctx.args.len() != 1 {
            return Err(ExecutionError::invalid_argument_count(1, fctx.args.len()));
        }

        Ok(cel_interpreter::Value::Bool(fctx.ptx.resolve(&fctx.args[0]).is_ok()))
    }
}
