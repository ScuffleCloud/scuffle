use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub struct All;

fn native_impl(iter: TokenStream, item_ident: syn::Ident, compare: impl ToTokens) -> syn::Expr {
    parse_quote!({
        let mut iter = (#iter).into_iter();
        loop {
            let Some(#item_ident) = iter.next() else {
                break true;
            };

            if !(#compare) {
                break false;
            }
        }
    })
}

// this.all(<ident>, <expr>)
impl Function for All {
    fn name(&self) -> &'static str {
        "all"
    }

    fn syntax(&self) -> &'static str {
        "<this>.all(<ident>, <expr>)"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = &ctx.this else {
            return Err(CompileError::syntax("missing this", self));
        };

        if ctx.args.len() != 2 {
            return Err(CompileError::syntax("invalid number of args", self));
        }

        let cel_parser::Expression::Ident(variable) = &ctx.args[0] else {
            return Err(CompileError::syntax("first argument must be an ident", self));
        };

        match this {
            CompiledExpr::Runtime(RuntimeCompiledExpr { expr, ty }) => {
                let mut child_ctx = ctx.child();

                match ty {
                    CelType::CelValue => {
                        child_ctx.add_variable(variable, CompiledExpr::runtime(CelType::CelValue, parse_quote!(item)));
                    }
                    CelType::Proto(ProtoType::Modified(
                        ProtoModifiedValueType::Repeated(ty) | ProtoModifiedValueType::Map(ty, _),
                    )) => {
                        child_ctx.add_variable(
                            variable,
                            CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ty.clone())), parse_quote!(item)),
                        );
                    }
                    v => {
                        return Err(CompileError::TypeConversion {
                            ty: Box::new(v.clone()),
                            message: "type cannot be iterated over".to_string(),
                        });
                    }
                };

                let arg = child_ctx.resolve(&ctx.args[1])?.to_bool(&child_ctx);

                Ok(CompiledExpr::runtime(
                    CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                    match &ty {
                        CelType::CelValue => parse_quote! {
                            ::tinc::__private::cel::CelValue::cel_all(#expr, |item| {
                                ::core::result::Result::Ok(
                                    #arg
                                )
                            })
                        },
                        CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(_, _))) => {
                            native_impl(quote!((#expr).keys()), parse_quote!(item), arg)
                        }
                        CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(_))) => {
                            native_impl(quote!((#expr).iter()), parse_quote!(item), arg)
                        }
                        _ => unreachable!(),
                    },
                ))
            }
            CompiledExpr::Constant(ConstantCompiledExpr {
                value: value @ (CelValue::List(_) | CelValue::Map(_)),
            }) => {
                let compile_val = |value: CelValue<'static>| {
                    let mut child_ctx = ctx.child();

                    child_ctx.add_variable(variable, CompiledExpr::constant(value));

                    child_ctx.resolve(&ctx.args[1]).map(|v| v.to_bool(&child_ctx))
                };

                let collected: Result<Vec<_>, _> = match value {
                    CelValue::List(item) => item.iter().cloned().map(compile_val).collect(),
                    CelValue::Map(item) => item.iter().map(|(key, _)| key).cloned().map(compile_val).collect(),
                    _ => unreachable!(),
                };

                let collected = collected?;
                if collected.iter().any(|c| matches!(c, CompiledExpr::Runtime(_))) {
                    Ok(CompiledExpr::runtime(
                        CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                        native_impl(quote!([#(#collected),*]), parse_quote!(item), quote!(item)),
                    ))
                } else {
                    Ok(CompiledExpr::constant(CelValue::Bool(collected.into_iter().all(
                        |c| match c {
                            CompiledExpr::Constant(ConstantCompiledExpr { value }) => value.to_bool(),
                            _ => unreachable!("all values must be constant"),
                        },
                    ))))
                }
            }
            CompiledExpr::Constant(ConstantCompiledExpr { value }) => Err(CompileError::TypeConversion {
                ty: Box::new(CelType::CelValue),
                message: format!("{value:?} cannot be iterated over"),
            }),
        }
    }
}
