use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub(crate) struct Map;

fn native_impl(iter: TokenStream, item_ident: syn::Ident, map_fn: impl ToTokens) -> syn::Expr {
    parse_quote!({
        let mut collected = Vec::new();
        let mut iter = (#iter).into_iter();
        loop {
            let Some(#item_ident) = iter.next() else {
                break ::tinc::__private::cel::CelValue::List(collected.into());
            };

            collected.push(#map_fn);
        }
    })
}

// this.map(<ident>, <expr>)
impl Function for Map {
    fn name(&self) -> &'static str {
        "map"
    }

    fn syntax(&self) -> &'static str {
        "<this>.map(<ident>, <expr>)"
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

                let arg = child_ctx.resolve(&ctx.args[1])?.into_cel()?;

                Ok(CompiledExpr::runtime(
                    CelType::CelValue,
                    match &ty {
                        CelType::CelValue => parse_quote! {
                            ::tinc::__private::cel::CelValue::cel_map(#expr, |item| {
                                ::core::result::Result::Ok(
                                    #arg
                                )
                            })?
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

                    child_ctx.resolve(&ctx.args[1])?.into_cel()
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
                    Ok(CompiledExpr::constant(CelValue::List(
                        collected
                            .into_iter()
                            .map(|c| match c {
                                CompiledExpr::Constant(ConstantCompiledExpr { value }) => value,
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
    use crate::codegen::cel::functions::{Function, Map};
    use crate::codegen::cel::types::CelType;
    use crate::types::{ProtoModifiedValueType, ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_map_syntax() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(Map.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.map(<ident>, <expr>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Map.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("hi".into()))), &[])), @r#"
        Err(
            InvalidSyntax {
                message: "invalid number of args",
                syntax: "<this>.map(<ident>, <expr>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Map.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("hi".into()))), &[
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

        insta::assert_debug_snapshot!(Map.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::Bool)), parse_quote!(input))), &[
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

        insta::assert_debug_snapshot!(Map.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List([
            CelValueConv::conv(0),
            CelValueConv::conv(1),
            CelValueConv::conv(-50),
            CelValueConv::conv(50),
        ].into_iter().collect()))), &[
            cel_parser::parse("x").unwrap(),
            cel_parser::parse("x + 1").unwrap(),
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
                                    2,
                                ),
                            ),
                            Number(
                                I64(
                                    -49,
                                ),
                            ),
                            Number(
                                I64(
                                    51,
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

        insta::assert_debug_snapshot!(Map.compile(CompilerCtx::new(ctx, Some(input), &[
            cel_parser::parse("x").unwrap(),
            cel_parser::parse("input[x]").unwrap(),
        ])), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: List(
                        [
                            Number(
                                I64(
                                    0,
                                ),
                            ),
                            Number(
                                I64(
                                    1,
                                ),
                            ),
                            Number(
                                I64(
                                    -50,
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
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_map_runtime_map() {
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

        let output = Map
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[cel_parser::parse("x").unwrap(), cel_parser::parse("input[x] * 2").unwrap()],
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
                        map.insert("2".to_string(), -50);
                        map.insert("3".to_string(), 50);
                        map
                    }).unwrap(), ::tinc::__private::cel::CelValue::List([
                        ::tinc::__private::cel::CelValueConv::conv(0),
                        ::tinc::__private::cel::CelValueConv::conv(2),
                        ::tinc::__private::cel::CelValueConv::conv(-100),
                        ::tinc::__private::cel::CelValueConv::conv(100),
                    ].into_iter().collect()));
                }
            },
        ));
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_map_runtime_repeated() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let string_value = CompiledExpr::runtime(
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(ProtoValueType::Int32))),
            parse_quote!(input),
        );

        let output = Map
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[cel_parser::parse("x").unwrap(), cel_parser::parse("x * 100").unwrap()],
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
                        ::tinc::__private::cel::CelValueConv::conv(0),
                        ::tinc::__private::cel::CelValueConv::conv(100),
                        ::tinc::__private::cel::CelValueConv::conv(-5000),
                        ::tinc::__private::cel::CelValueConv::conv(5000),
                    ].into_iter().collect()));
                }
            },
        ));
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_map_runtime_cel_value() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let string_value = CompiledExpr::runtime(CelType::CelValue, parse_quote!(input));

        let output = Map
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[cel_parser::parse("x").unwrap(), cel_parser::parse("x + 1").unwrap()],
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
                        tinc::__private::cel::CelValueConv::conv(6),
                        tinc::__private::cel::CelValueConv::conv(2),
                        tinc::__private::cel::CelValueConv::conv(51),
                         tinc::__private::cel::CelValueConv::conv(-49),
                    ].into_iter().collect()));
                }
            },
        ));
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_map_const_requires_runtime() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let list_value = CompiledExpr::constant(CelValue::List(
            [CelValueConv::conv(5), CelValueConv::conv(0), CelValueConv::conv(1)]
                .into_iter()
                .collect(),
        ));

        let output = Map
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(list_value),
                &[cel_parser::parse("x").unwrap(), cel_parser::parse("dyn(x / 2)").unwrap()],
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
                        ::tinc::__private::cel::CelValueConv::conv(2),
                        ::tinc::__private::cel::CelValueConv::conv(0),
                        ::tinc::__private::cel::CelValueConv::conv(0),
                    ].into_iter().collect()));
                }
            },
        ));
    }
}
