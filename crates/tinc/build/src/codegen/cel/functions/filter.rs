use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub struct Filter;

fn native_impl(iter: TokenStream, item_ident: syn::Ident, compare: impl ToTokens) -> syn::Expr {
    parse_quote!({
        let mut collected = Vec::new();
        let mut iter = (#iter).into_iter();
        loop {
            let Some(#item_ident) = iter.next() else {
                break ::tinc::__private::cel::CelValue::List(collected.into());
            };

            if {
                let #item_ident = #item_ident.clone()
                #compare
            } {
                colleced.push(#item_ident);
            }
        }
    })
}

// this.filter(<ident>, <expr>)
impl Function for Filter {
    fn name(&self) -> &'static str {
        "filter"
    }

    fn syntax(&self) -> &'static str {
        "<this>.filter(<ident>, <expr>)"
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
                    CelType::CelValue,
                    match ty {
                        CelType::CelValue => parse_quote! {
                            ::tinc::__private::cel::CelValue::cel_filter(#expr, |item| {
                                ::core::result::Result::Ok(
                                    #arg
                                )
                            })
                        },
                        CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(ty, _))) => {
                            let cel_ty =
                                CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ty.clone())), parse_quote!(item))
                                    .to_cel()?;

                            native_impl(
                                quote!(
                                    (#expr).keys().map(|item| #cel_ty)
                                ),
                                parse_quote!(item),
                                arg,
                            )
                        }
                        CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(ty))) => {
                            let cel_ty =
                                CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ty.clone())), parse_quote!(item))
                                    .to_cel()?;

                            native_impl(
                                quote!(
                                    (#expr).iter().map(|item| #cel_ty)
                                ),
                                parse_quote!(item),
                                arg,
                            )
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

                    child_ctx.add_variable(variable, CompiledExpr::constant(value.clone()));

                    child_ctx.resolve(&ctx.args[1]).map(|v| (value, v.to_bool(&child_ctx)))
                };

                let collected: Result<Vec<_>, _> = match value {
                    CelValue::List(item) => item.iter().cloned().map(compile_val).collect(),
                    CelValue::Map(item) => item.iter().map(|(key, _)| key).cloned().map(compile_val).collect(),
                    _ => unreachable!(),
                };

                let collected = collected?;
                if collected.iter().any(|(_, c)| matches!(c, CompiledExpr::Runtime(_))) {
                    let collected = collected.into_iter().map(|(item, expr)| {
                        let item = CompiledExpr::constant(item);
                        quote! {
                            if #expr {
                                collected.push(#item);
                            }
                        }
                    });

                    Ok(CompiledExpr::runtime(
                        CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                        parse_quote!({
                            let mut collected = Vec::new();
                            #(#collected)*
                            ::tinc::__private::cel::CelValue::List(collected.into());
                        }),
                    ))
                } else {
                    Ok(CompiledExpr::constant(CelValue::List(
                        collected
                            .into_iter()
                            .filter_map(|(item, c)| match c {
                                CompiledExpr::Constant(ConstantCompiledExpr { value }) => {
                                    if value.to_bool() {
                                        Some(item)
                                    } else {
                                        None
                                    }
                                }
                                _ => unreachable!("all values must be constant"),
                            })
                            .collect(),
                    )))
                }
            }
            CompiledExpr::Constant(ConstantCompiledExpr { value }) => Err(CompileError::TypeConversion {
                ty: Box::new(CelType::CelValue),
                message: format!("{value:?} cannot be iterated over"),
            }),
        }
    }
}
