use cel_interpreter::{ExecutionError, FunctionContext};
use syn::parse_quote;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoType, ProtoValueType};

pub struct Matches;

// this.matches(arg) -> arg in this
impl Function for Matches {
    const NAME: &'static str = "matches";

    fn compile(ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = &ctx.this else {
            return Err(CompileError::MissingTarget {
                func: Self::NAME,
                message: "this is required when calling the matches function".to_string(),
            });
        };

        if ctx.args.len() != 1 {
            return Err(CompileError::InvalidFunctionArgumentCount {
                func: Self::NAME,
                expected: 1,
                got: ctx.args.len(),
            });
        }

        let cel_parser::Expression::Atom(cel_parser::Atom::String(regex)) = &ctx.args[0] else {
            return Err(CompileError::InvalidFunctionArgument {
                message: "the regex expression must be known at compile time".into(),
                expr: ctx.args[0].clone(),
                idx: 0,
            });
        };

        let regex = regex.as_str();

        if let Err(err) = regex::Regex::new(&regex) {
            return Err(CompileError::InvalidFunctionArgument {
                message: format!("bad regex expression: {err}"),
                expr: ctx.args[0].clone(),
                idx: 0,
            });
        }

        if !this.ty.can_be_cel() {
            return Err(CompileError::TypeConversion {
                ty: Box::new(this.ty.clone()),
                message: "the matches function can only be called with CEL value argument types".to_string(),
            });
        }

        Ok(CompiledExpr {
            expr: parse_quote! {{
                static REGEX: ::std::sync::LazyLock<::tinc::reexports::regex::Regex> = ::std::sync::LazyLock::new(|| {
                    regex::Regex::new(#regex).expect("regex failed to compile")
                })

                ::tinc::__private::cel::CelValue::cel_matches(
                    #this,
                    &*REGEX,
                )?
            }},
            ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
        })
    }

    fn interpret(fctx: &FunctionContext) -> Result<cel_interpreter::Value, ExecutionError> {
        let Some(cel_interpreter::Value::String(this)) = &fctx.this else {
            return Err(ExecutionError::missing_argument_or_target());
        };

        if fctx.args.len() != 1 {
            return Err(ExecutionError::invalid_argument_count(1, fctx.args.len()));
        }

        let arg = fctx.ptx.resolve(&fctx.args[0])?;
        let regex = regex::Regex::new(this).map_err(|err| ExecutionError::FunctionError {
            function: Self::NAME.to_owned(),
            message: format!("bad regex: {err}"),
        })?;

        match arg {
            cel_interpreter::Value::String(s) => Ok(cel_interpreter::Value::Bool(regex.is_match(&s))),
            cel_interpreter::Value::Bytes(t) => {
                if let Ok(s) = std::str::from_utf8(&t) {
                    Ok(cel_interpreter::Value::Bool(regex.is_match(s)))
                } else {
                    Ok(cel_interpreter::Value::Bool(false))
                }
            }
            _ => Ok(cel_interpreter::Value::Bool(false)),
        }
    }
}
