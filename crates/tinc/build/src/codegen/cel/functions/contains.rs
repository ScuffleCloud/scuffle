use quote::quote;
use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub(crate) struct Contains;

// this.contains(arg)
// arg in this
impl Function for Contains {
    fn name(&self) -> &'static str {
        "contains"
    }

    fn syntax(&self) -> &'static str {
        "<this>.contains(<arg>)"
    }

    fn compile(&self, mut ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = ctx.this.take() else {
            return Err(CompileError::syntax("missing this", self));
        };

        if ctx.args.len() != 1 {
            return Err(CompileError::syntax("takes exactly one argument", self));
        }

        let arg = ctx.resolve(&ctx.args[0])?.into_cel()?;

        if let CompiledExpr::Runtime(RuntimeCompiledExpr {
            expr,
            ty:
                ty @ CelType::Proto(ProtoType::Modified(
                    ProtoModifiedValueType::Repeated(item) | ProtoModifiedValueType::Map(item, _),
                )),
        }) = &this
        {
            if !matches!(item, ProtoValueType::Message { .. } | ProtoValueType::Enum(_)) {
                let op = match &ty {
                    CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(_))) => {
                        quote! { array_contains }
                    }
                    CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(_, _))) => {
                        quote! { map_contains }
                    }
                    _ => unreachable!(),
                };

                return Ok(CompiledExpr::runtime(
                    CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                    parse_quote! {
                        ::tinc::__private::cel::#op(
                            #expr,
                            #arg,
                        )
                    },
                ));
            }
        }

        let this = this.clone().into_cel()?;

        match (this, arg) {
            (
                CompiledExpr::Constant(ConstantCompiledExpr { value: this }),
                CompiledExpr::Constant(ConstantCompiledExpr { value: arg }),
            ) => Ok(CompiledExpr::constant(CelValue::cel_contains(this, arg)?)),
            (this, arg) => Ok(CompiledExpr::runtime(
                CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                parse_quote! {
                    ::tinc::__private::cel::CelValue::cel_contains(
                        #this,
                        #arg,
                    )?
                },
            )),
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use quote::quote;
    use syn::parse_quote;
    use tinc_cel::CelValue;

    use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerCtx};
    use crate::codegen::cel::functions::{Contains, Function};
    use crate::codegen::cel::types::CelType;
    use crate::types::{ProtoModifiedValueType, ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_contains_syntax() {
        let registry = ProtoTypeRegistry::new();
        let compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(Contains.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.contains(<arg>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Contains.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("hi".into()))), &[])), @r#"
        Err(
            InvalidSyntax {
                message: "takes exactly one argument",
                syntax: "<this>.contains(<arg>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Contains.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List(Default::default()))), &[
            cel_parser::parse("1 + 1").unwrap(),
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
    fn test_contains_runtime_string() {
        let registry = ProtoTypeRegistry::new();
        let compiler = Compiler::new(&registry);

        let string_value =
            CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::String)), parse_quote!(input));

        let output = Contains
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[cel_parser::parse("(1 + 1).string()").unwrap()],
            ))
            .unwrap();

        insta::assert_snapshot!(postcompile::compile_str!(
            postcompile::config! {
                test: true,
                dependencies: vec![
                    postcompile::Dependency::workspace("tinc"),
                ],
            },
            quote! {
                fn contains(input: &String) -> Result<bool, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_contains() {
                    assert_eq!(contains(&"in2dastring".into()).unwrap(), true);
                    assert_eq!(contains(&"in3dastring".into()).unwrap(), false);
                }
            },
        ));
    }

    #[test]
    fn test_contains_runtime_map() {
        let registry = ProtoTypeRegistry::new();
        let compiler = Compiler::new(&registry);

        let string_value = CompiledExpr::runtime(
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(
                ProtoValueType::String,
                ProtoValueType::Bool,
            ))),
            parse_quote!(input),
        );

        let output = Contains
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[cel_parser::parse("'value'").unwrap()],
            ))
            .unwrap();

        insta::assert_snapshot!(postcompile::compile_str!(
            postcompile::config! {
                test: true,
                dependencies: vec![
                    postcompile::Dependency::workspace("tinc"),
                ],
            },
            quote! {
                fn contains(input: &std::collections::HashMap<String, bool>) -> Result<bool, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_contains() {
                    assert_eq!(contains(&{
                        let mut map = std::collections::HashMap::new();
                        map.insert("value".to_string(), true);
                        map
                    }).unwrap(), true);
                    assert_eq!(contains(&{
                        let mut map = std::collections::HashMap::new();
                        map.insert("not_value".to_string(), true);
                        map
                    }).unwrap(), false);
                    assert_eq!(contains(&{
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
    fn test_contains_runtime_repeated() {
        let registry = ProtoTypeRegistry::new();
        let compiler = Compiler::new(&registry);

        let string_value = CompiledExpr::runtime(
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(ProtoValueType::String))),
            parse_quote!(input),
        );

        let output = Contains
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[cel_parser::parse("'value'").unwrap()],
            ))
            .unwrap();

        insta::assert_snapshot!(postcompile::compile_str!(
            postcompile::config! {
                test: true,
                dependencies: vec![
                    postcompile::Dependency::workspace("tinc"),
                ],
            },
            quote! {
                fn contains(input: &Vec<String>) -> Result<bool, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_contains() {
                    assert_eq!(contains(&vec!["value".into()]).unwrap(), true);
                    assert_eq!(contains(&vec!["not_value".into()]).unwrap(), false);
                    assert_eq!(contains(&vec!["xd".into(), "value".into()]).unwrap(), true);
                }
            },
        ));
    }
}
