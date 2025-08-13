use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub(crate) struct Exists;

fn native_impl(iter: TokenStream, item_ident: syn::Ident, compare: impl ToTokens) -> syn::Expr {
    parse_quote!({
        let mut iter = (#iter).into_iter();
        loop {
            let Some(#item_ident) = iter.next() else {
                break false;
            };

            if #compare {
                break true;
            }
        }
    })
}

// this.exists(<ident>, <expr>)
impl Function for Exists {
    fn name(&self) -> &'static str {
        "exists"
    }

    fn syntax(&self) -> &'static str {
        "<this>.exists(<ident>, <expr>)"
    }

    fn compile(&self, mut ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = ctx.this.take() else {
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

                match &ty {
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
                    CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                    match &ty {
                        CelType::CelValue => parse_quote! {
                            ::tinc::__private::cel::CelValue::cel_exists(#expr, |item| {
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

                    child_ctx.resolve(&ctx.args[1]).map(|v| v.into_bool(&child_ctx))
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
                    Ok(CompiledExpr::constant(CelValue::Bool(collected.into_iter().any(
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

#[cfg(test)]
#[cfg(feature = "prost")]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use quote::quote;
    use syn::parse_quote;
    use tinc_cel::{CelValue, CelValueConv};

    use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerCtx};
    use crate::codegen::cel::functions::{Exists, Function};
    use crate::codegen::cel::types::CelType;
    use crate::types::{ProtoModifiedValueType, ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_exists_syntax() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(Exists.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.exists(<ident>, <expr>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Exists.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("hi".into()))), &[])), @r#"
        Err(
            InvalidSyntax {
                message: "invalid number of args",
                syntax: "<this>.exists(<ident>, <expr>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Exists.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("hi".into()))), &[
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

        insta::assert_debug_snapshot!(Exists.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::Bool)), parse_quote!(input))), &[
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

        insta::assert_debug_snapshot!(Exists.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List(Default::default()))), &[
            cel_parser::parse("x").unwrap(),
            cel_parser::parse("x == 'value'").unwrap(),
        ])), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Bool(
                        false,
                    ),
                },
            ),
        )
        ");
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_exists_runtime_map() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let string_value = CompiledExpr::runtime(
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(
                ProtoValueType::String,
                ProtoValueType::Bool,
            ))),
            parse_quote!(input),
        );

        let output = Exists
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[cel_parser::parse("x").unwrap(), cel_parser::parse("x == 'value'").unwrap()],
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
                fn exists(input: &std::collections::HashMap<String, bool>) -> Result<bool, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_contains() {
                    assert_eq!(exists(&{
                        let mut map = std::collections::HashMap::new();
                        map.insert("value".to_string(), true);
                        map
                    }).unwrap(), true);
                    assert_eq!(exists(&{
                        let mut map = std::collections::HashMap::new();
                        map.insert("not_value".to_string(), true);
                        map
                    }).unwrap(), false);
                    assert_eq!(exists(&{
                        let mut map = std::collections::HashMap::new();
                        map.insert("xd".to_string(), true);
                        map.insert("value".to_string(), true);
                        map
                    }).unwrap(), true);
                }
            },
        ));
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_exists_runtime_repeated() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let string_value = CompiledExpr::runtime(
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(ProtoValueType::String))),
            parse_quote!(input),
        );

        let output = Exists
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[cel_parser::parse("x").unwrap(), cel_parser::parse("x == 'value'").unwrap()],
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
                fn exists(input: &Vec<String>) -> Result<bool, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_exists() {
                    assert_eq!(exists(&vec!["value".into()]).unwrap(), true);
                    assert_eq!(exists(&vec!["not_value".into()]).unwrap(), false);
                    assert_eq!(exists(&vec!["xd".into(), "value".into()]).unwrap(), true);
                    assert_eq!(exists(&vec!["xd".into(), "value".into(), "value".into()]).unwrap(), true);
                }
            },
        ));
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_exists_runtime_cel_value() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let string_value = CompiledExpr::runtime(CelType::CelValue, parse_quote!(input));

        let output = Exists
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[cel_parser::parse("x").unwrap(), cel_parser::parse("x == 'value'").unwrap()],
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
                fn exists<'a>(input: &'a ::tinc::__private::cel::CelValue<'a>) -> Result<bool, ::tinc::__private::cel::CelError<'a>> {
                    Ok(#output)
                }

                #[test]
                fn test_exists() {
                    assert_eq!(exists(&tinc::__private::cel::CelValue::List([
                        tinc::__private::cel::CelValueConv::conv("value"),
                    ].into_iter().collect())).unwrap(), true);
                    assert_eq!(exists(&tinc::__private::cel::CelValue::List([
                        tinc::__private::cel::CelValueConv::conv("not_value"),
                    ].into_iter().collect())).unwrap(), false);
                    assert_eq!(exists(&tinc::__private::cel::CelValue::List([
                        tinc::__private::cel::CelValueConv::conv("xd"),
                        tinc::__private::cel::CelValueConv::conv("value"),
                    ].into_iter().collect())).unwrap(), true);
                    assert_eq!(exists(&tinc::__private::cel::CelValue::List([
                        tinc::__private::cel::CelValueConv::conv("xd"),
                        tinc::__private::cel::CelValueConv::conv("value"),
                        tinc::__private::cel::CelValueConv::conv("value"),
                    ].into_iter().collect())).unwrap(), true);
                }
            },
        ));
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_exists_const_requires_runtime() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let list_value = CompiledExpr::constant(CelValue::List(
            [CelValueConv::conv(5), CelValueConv::conv(0), CelValueConv::conv(1)]
                .into_iter()
                .collect(),
        ));

        let output = Exists
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
                fn exists_one() -> Result<bool, ::tinc::__private::cel::CelError<'static>> {
                    Ok(#output)
                }

                #[test]
                fn test_filter() {
                    assert_eq!(exists_one().unwrap(), true);
                }
            },
        ));
    }
}
