use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub(crate) struct All;

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
            return Err(CompileError::syntax("invalid number of args, expected 2", self));
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use indexmap::IndexMap;
    use quote::{format_ident, quote};
    use syn::parse_quote;
    use tinc_cel::{CelValue, CelValueConv};

    use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerCtx};
    use crate::codegen::cel::functions::{All, Function};
    use crate::codegen::cel::types::CelType;
    use crate::types::{
        ProtoEnumOptions, ProtoEnumType, ProtoEnumVariant, ProtoEnumVariantOptions, ProtoModifiedValueType, ProtoPath,
        ProtoType, ProtoTypeRegistry, ProtoValueType, ProtoVisibility,
    };

    #[test]
    fn test_all_syntax() {
        let registry = ProtoTypeRegistry::new();
        let compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(All.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.all(<ident>, <expr>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(All.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List(Default::default()))), &[])), @r#"
        Err(
            InvalidSyntax {
                message: "invalid number of args, expected 2",
                syntax: "<this>.all(<ident>, <expr>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(All.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List(Default::default()))), &[
            cel_parser::parse("1 + 1").unwrap(), // not an ident
            cel_parser::parse("x + 2").unwrap(),
        ])), @r#"
        Err(
            InvalidSyntax {
                message: "first argument must be an ident",
                syntax: "<this>.all(<ident>, <expr>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(All.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List([
            CelValueConv::conv(4),
            CelValueConv::conv(3),
            CelValueConv::conv(10),
        ].into_iter().collect()))), &[
            cel_parser::parse("x").unwrap(),
            cel_parser::parse("x > 2").unwrap(),
        ])), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Bool(
                        true,
                    ),
                },
            ),
        )
        ");

        insta::assert_debug_snapshot!(All.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List([
            CelValueConv::conv(2),
        ].into_iter().collect()))), &[
            cel_parser::parse("x").unwrap(),
            cel_parser::parse("x > 2").unwrap(),
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

        insta::assert_debug_snapshot!(All.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::Map([
            (CelValueConv::conv(2), CelValue::Null),
        ].into_iter().collect()))), &[
            cel_parser::parse("x").unwrap(),
            cel_parser::parse("x > 2").unwrap(),
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

        insta::assert_debug_snapshot!(All.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValueConv::conv(1))), &[
            cel_parser::parse("x").unwrap(),
            cel_parser::parse("x > 2").unwrap(),
        ])), @r#"
        Err(
            TypeConversion {
                ty: CelValue,
                message: "Number(I64(1)) cannot be iterated over",
            },
        )
        "#);
    }

    #[test]
    fn test_all_proto_map() {
        let registry = ProtoTypeRegistry::new();
        let compiler = Compiler::new(&registry);

        let map = CompiledExpr::runtime(
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(
                ProtoValueType::Int32,
                ProtoValueType::Float,
            ))),
            parse_quote!(input),
        );

        let result = All
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(map),
                &[
                    cel_parser::parse("x").unwrap(), // not an ident
                    cel_parser::parse("x > 2").unwrap(),
                ],
            ))
            .unwrap();

        let small_fn = quote! {
            #[allow(dead_code)]
            fn hash_map(input: &std::collections::HashMap<i32, f64>) -> Result<bool, ::tinc::__private::cel::CelError<'_>> {
                Ok(#result)
            }

            #[allow(dead_code)]
            fn btree_map(input: &std::collections::BTreeMap<i32, f64>) -> Result<bool, ::tinc::__private::cel::CelError<'_>> {
                Ok(#result)
            }
        };

        let compiled = postcompile::compile_str!(&small_fn.to_string());
        insta::assert_snapshot!(compiled);
    }

    #[test]
    fn test_all_proto_repeated() {
        let registry = ProtoTypeRegistry::new();
        let compiler = Compiler::new(&registry);

        let repeated = CompiledExpr::runtime(
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(ProtoValueType::Int32))),
            parse_quote!(input),
        );

        let result = All
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(repeated),
                &[
                    cel_parser::parse("x").unwrap(), // not an ident
                    cel_parser::parse("x > 2").unwrap(),
                ],
            ))
            .unwrap();

        let small_fn = quote! {
            #[allow(dead_code)]
            fn vec(input: &Vec<i32>) -> Result<bool, ::tinc::__private::cel::CelError<'_>> {
                Ok(#result)
            }

            #[allow(dead_code)]
            fn slice(input: &[i32]) -> Result<bool, ::tinc::__private::cel::CelError<'_>> {
                Ok(#result)
            }
        };

        let compiled = postcompile::compile_str!(&small_fn.to_string());
        insta::assert_snapshot!(compiled);
    }

    #[test]
    fn test_all_const_needs_runtime() {
        let mut registry = ProtoTypeRegistry::new();
        registry.register_enum(ProtoEnumType {
            full_name: ProtoPath::new("some.UnknownEnum"),
            options: ProtoEnumOptions { repr_enum: false },
            package: ProtoPath::new("some"),
            variants: {
                let mut map = IndexMap::new();

                map.insert(
                    "FIRST_VALUE".into(),
                    ProtoEnumVariant {
                        full_name: ProtoPath::new("some.UnknownEnum.FIRST_VALUE"),
                        value: 0,
                        options: ProtoEnumVariantOptions {
                            json_name: "FirstValue".into(),
                            visibility: ProtoVisibility::Default,
                        },
                        rust_ident: format_ident!("FirstValue"),
                    },
                );

                map
            },
        });

        let compiler = Compiler::new(&registry);

        let list = CompiledExpr::constant(CelValue::List([CelValue::Number(0.into())].into_iter().collect()));

        let result = All
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(list),
                &[
                    cel_parser::parse("x").unwrap(), // not an ident
                    cel_parser::parse("x.enum('some.UnknownEnum').string()").unwrap(),
                ],
            ))
            .unwrap();

        let small_fn = quote! {
            #[allow(dead_code)]
            fn dyn_func() -> Result<bool, ::tinc::__private::cel::CelError<'static>> {
                Ok(#result)
            }
        };

        let compiled = postcompile::compile_str!(&small_fn.to_string());
        insta::assert_snapshot!(compiled);
    }
}
