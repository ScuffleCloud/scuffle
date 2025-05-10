use std::collections::HashMap;

use anyhow::Context;
use compiler::{CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use functions::Function;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use tinc_cel::CelValue;

pub(crate) mod compiler;
pub(crate) mod functions;
pub(crate) mod types;

pub(crate) fn eval_message_fmt(
    field_full_name: &str,
    msg: &str,
    ctx: &compiler::Compiler<'_>,
) -> anyhow::Result<TokenStream> {
    let fmt = runtime_format::ParsedFmt::new(msg).map_err(|err| anyhow::anyhow!("failed to parse message format: {err}"))?;

    let mut runtime_args = Vec::new();
    let mut compile_time_args = HashMap::new();

    // each key itself a cel expression
    for key in fmt.keys() {
        let expr = cel_parser::parse(key).context("failed to parse cel expression")?;
        match functions::String.compile(CompilerCtx::new(ctx.child(), Some(ctx.resolve(&expr)?), &[]))? {
            CompiledExpr::Constant(ConstantCompiledExpr { value }) => {
                // we need to escape the '{' & '}'
                compile_time_args.insert(key, value);
            }
            CompiledExpr::Runtime(RuntimeCompiledExpr { expr, .. }) => {
                let ident = format_ident!("arg_{}", runtime_args.len());
                compile_time_args.insert(key, CelValue::String(format!("{{{ident}}}").into()));
                runtime_args.push((key, ident, expr));
            }
        }
    }

    let fmt = fmt.with_args(&compile_time_args).to_string();

    if runtime_args.is_empty() {
        Ok(quote!(#fmt))
    } else {
        let args = runtime_args.iter().map(|(key, ident, expr)| {
            quote! {
                #ident = (
                    || {
                        ::core::result::Result::Ok::<_, ::tinc::__private::cel::CelError>(#expr)
                    }
                )().map_err(|err| {
                    ::tinc::__private::ValidationError::Expression {
                        error: err.to_string().into_boxed_str(),
                        field: #field_full_name,
                        expression: #key,
                    }
                })?
            }
        });

        Ok(quote!(format!(#fmt, #(#args),*)))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CelExpression {
    pub message: String,
    pub expression: String,
    pub jsonschemas: Vec<String>,
    pub this: Option<CelValue<'static>>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub(crate) struct CelExpressions {
    pub field: Vec<CelExpression>,
    pub map_key: Vec<CelExpression>,
    pub map_value: Vec<CelExpression>,
    pub repeated_item: Vec<CelExpression>,
}
