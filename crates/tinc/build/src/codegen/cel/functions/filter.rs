use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub(crate) struct Filter;

fn native_impl(iter: TokenStream, item_ident: syn::Ident, compare: impl ToTokens) -> syn::Expr {
    parse_quote!({
        let mut collected = Vec::new();
        let mut iter = (#iter).into_iter();
        loop {
            let Some(#item_ident) = iter.next() else {
                break ::tinc::__private::cel::CelValue::List(collected.into());
            };

            if {
                let #item_ident = #item_ident.clone();
                #compare
            } {
                collected.push(#item_ident);
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

                let arg = child_ctx.resolve(&ctx.args[1])?.into_bool(&child_ctx);

                Ok(CompiledExpr::runtime(
                    CelType::CelValue,
                    match ty {
                        CelType::CelValue => parse_quote! {
                            ::tinc::__private::cel::CelValue::cel_filter(#expr, |item| {
                                ::core::result::Result::Ok(
                                    #arg
                                )
                            })?
                        },
                        CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(ty, _))) => {
                            let cel_ty =
                                CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ty.clone())), parse_quote!(item))
                                    .into_cel()?;

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
                                    .into_cel()?;

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

                    child_ctx.resolve(&ctx.args[1]).map(|v| (value, v.into_bool(&child_ctx)))
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
                            ::tinc::__private::cel::CelValue::List(collected.into())
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

#[cfg(test)]
#[cfg(feature = "prost")]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use quote::quote;
    use syn::parse_quote;
    use tinc_cel::{CelValue, CelValueConv};

    use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerCtx};
    use crate::codegen::cel::functions::{Filter, Function};
    use crate::codegen::cel::types::CelType;
    use crate::types::{ProtoModifiedValueType, ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_filter_syntax() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(Filter.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.filter(<ident>, <expr>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Filter.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("hi".into()))), &[])), @r#"
        Err(
            InvalidSyntax {
                message: "invalid number of args",
                syntax: "<this>.filter(<ident>, <expr>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Filter.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("hi".into()))), &[
            cel_parser::parse("x").unwrap(),
            cel_parser::parse("dyn(x >= 1)").unwrap(),
        ])), @r#"
        Err(
            TypeConversion {
                ty: CelValue,
                message: "String(Borrowed(\"hi\")) cannot be iterated over",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Filter.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::Bool)), parse_quote!(input))), &[
            cel_parser::parse("x").unwrap(),
            cel_parser::parse("dyn(x >= 1)").unwrap(),
        ])), @r#"
        Err(
            TypeConversion {
                ty: Proto(
                    Value(
                        Bool,
                    ),
                ),
                message: "type cannot be iterated over",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Filter.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List([
            CelValueConv::conv(0),
            CelValueConv::conv(1),
            CelValueConv::conv(-50),
            CelValueConv::conv(50),
        ].into_iter().collect()))), &[
            cel_parser::parse("x").unwrap(),
            cel_parser::parse("x >= 1").unwrap(),
        ])), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: List(
                        [
                            Number(
                                I64(
                                    1,
                                ),
                            ),
                            Number(
                                I64(
                                    50,
                                ),
                            ),
                        ],
                    ),
                },
            ),
        )
        ");

        let input = CompiledExpr::constant(CelValue::Map(
            [
                (CelValueConv::conv("key0"), CelValueConv::conv(0)),
                (CelValueConv::conv("key1"), CelValueConv::conv(1)),
                (CelValueConv::conv("key2"), CelValueConv::conv(-50)),
                (CelValueConv::conv("key3"), CelValueConv::conv(50)),
            ]
            .into_iter()
            .collect(),
        ));

        let mut ctx = compiler.child();
        ctx.add_variable("input", input.clone());

        insta::assert_debug_snapshot!(Filter.compile(CompilerCtx::new(ctx, Some(input), &[
            cel_parser::parse("x").unwrap(),
            cel_parser::parse("input[x] >= 1").unwrap(),
        ])), @r#"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: List(
                        [
                            String(
                                Borrowed(
                                    "key1",
                                ),
                            ),
                            String(
                                Borrowed(
                                    "key3",
                                ),
                            ),
                        ],
                    ),
                },
            ),
        )
        "#);
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_filter_runtime_map() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let mut compiler = Compiler::new(&registry);

        let string_value = CompiledExpr::runtime(
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(
                ProtoValueType::String,
                ProtoValueType::Int32,
            ))),
            parse_quote!(input),
        );

        compiler.add_variable("input", string_value.clone());

        let output = Filter
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[cel_parser::parse("x").unwrap(), cel_parser::parse("input[x] >= 1").unwrap()],
            ))
            .unwrap();

        insta::assert_snapshot!(postcompile::compile_str!(
            postcompile::config! {
                test: true,
                dependencies: vec![
                    postcompile::Dependency::version("tinc", "*"),
                ],
            },
            quote! {
                fn filter(input: &std::collections::BTreeMap<String, i32>) -> Result<::tinc::__private::cel::CelValue<'_>, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_filter() {
                    assert_eq!(filter(&{
                        let mut map = std::collections::BTreeMap::new();
                        map.insert("0".to_string(), 0);
                        map.insert("1".to_string(), 1);
                        map.insert("-50".to_string(), -50);
                        map.insert("50".to_string(), 50);
                        map
                    }).unwrap(), ::tinc::__private::cel::CelValue::List([
                        ::tinc::__private::cel::CelValueConv::conv("1"),
                        ::tinc::__private::cel::CelValueConv::conv("50"),
                    ].into_iter().collect()));
                }
            },
        ));
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_filter_runtime_repeated() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let string_value = CompiledExpr::runtime(
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(ProtoValueType::Int32))),
            parse_quote!(input),
        );

        let output = Filter
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[cel_parser::parse("x").unwrap(), cel_parser::parse("x >= 1").unwrap()],
            ))
            .unwrap();

        insta::assert_snapshot!(postcompile::compile_str!(
            postcompile::config! {
                test: true,
                dependencies: vec![
                    postcompile::Dependency::version("tinc", "*"),
                ],
            },
            quote! {
                fn filter(input: &Vec<i32>) -> Result<::tinc::__private::cel::CelValue<'_>, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_filter() {
                    assert_eq!(filter(&vec![0, 1, -50, 50]).unwrap(), ::tinc::__private::cel::CelValue::List([
                        ::tinc::__private::cel::CelValueConv::conv(1),
                        ::tinc::__private::cel::CelValueConv::conv(50),
                    ].into_iter().collect()));
                }
            },
        ));
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_filter_runtime_cel_value() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let string_value = CompiledExpr::runtime(CelType::CelValue, parse_quote!(input));

        let output = Filter
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[cel_parser::parse("x").unwrap(), cel_parser::parse("x > 5").unwrap()],
            ))
            .unwrap();

        insta::assert_snapshot!(postcompile::compile_str!(
            postcompile::config! {
                test: true,
                dependencies: vec![
                    postcompile::Dependency::version("tinc", "*"),
                ],
            },
            quote! {
                fn filter<'a>(input: &'a ::tinc::__private::cel::CelValue<'a>) -> Result<::tinc::__private::cel::CelValue<'a>, ::tinc::__private::cel::CelError<'a>> {
                    Ok(#output)
                }

                #[test]
                fn test_filter() {
                    assert_eq!(filter(&tinc::__private::cel::CelValue::List([
                        tinc::__private::cel::CelValueConv::conv(5),
                        tinc::__private::cel::CelValueConv::conv(1),
                        tinc::__private::cel::CelValueConv::conv(50),
                         tinc::__private::cel::CelValueConv::conv(-50),
                    ].into_iter().collect())).unwrap(), tinc::__private::cel::CelValue::List([
                        tinc::__private::cel::CelValueConv::conv(50),
                    ].into_iter().collect()));
                }
            },
        ));
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_filter_const_requires_runtime() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let list_value = CompiledExpr::constant(CelValue::List(
            [CelValueConv::conv(5), CelValueConv::conv(0), CelValueConv::conv(1)]
                .into_iter()
                .collect(),
        ));

        let output = Filter
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(list_value),
                &[cel_parser::parse("x").unwrap(), cel_parser::parse("dyn(x >= 1)").unwrap()],
            ))
            .unwrap();

        insta::assert_snapshot!(postcompile::compile_str!(
            postcompile::config! {
                test: true,
                dependencies: vec![
                    postcompile::Dependency::version("tinc", "*"),
                ],
            },
            quote! {
                fn filter() -> Result<::tinc::__private::cel::CelValue<'static>, ::tinc::__private::cel::CelError<'static>> {
                    Ok(#output)
                }

                #[test]
                fn test_filter() {
                    assert_eq!(filter().unwrap(), ::tinc::__private::cel::CelValue::List([
                        ::tinc::__private::cel::CelValueConv::conv(5),
                        ::tinc::__private::cel::CelValueConv::conv(1),
                    ].into_iter().collect()));
                }
            },
        ));
    }
}
