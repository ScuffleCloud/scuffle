use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub(crate) struct Size;

impl Function for Size {
    fn name(&self) -> &'static str {
        "size"
    }

    fn syntax(&self) -> &'static str {
        "<this>.size()"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = ctx.this else {
            return Err(CompileError::syntax("missing this", self));
        };

        if !ctx.args.is_empty() {
            return Err(CompileError::syntax("takes no arguments", self));
        }

        if let CompiledExpr::Runtime(RuntimeCompiledExpr {
            expr,
            ty: CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(_) | ProtoModifiedValueType::Map(_, _))),
        }) = &this
        {
            return Ok(CompiledExpr::runtime(
                CelType::Proto(ProtoType::Value(ProtoValueType::UInt64)),
                parse_quote! {
                    ((#expr).len() as u64)
                },
            ));
        }

        match this.into_cel()? {
            CompiledExpr::Constant(ConstantCompiledExpr { value }) => Ok(CompiledExpr::constant(CelValue::cel_size(value)?)),
            CompiledExpr::Runtime(RuntimeCompiledExpr { expr, .. }) => Ok(CompiledExpr::runtime(
                CelType::Proto(ProtoType::Value(ProtoValueType::UInt64)),
                parse_quote!(::tinc::__private::cel::CelValue::cel_size(#expr)?),
            )),
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use syn::parse_quote;
    use tinc_cel::CelValue;

    use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerCtx};
    use crate::codegen::cel::functions::{Function, Size};
    use crate::codegen::cel::types::CelType;
    use crate::types::{ProtoModifiedValueType, ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_size_syntax() {
        let registry = ProtoTypeRegistry::new();
        let compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(Size.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.size()",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Size.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("13".into()))), &[])), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        U64(
                            2,
                        ),
                    ),
                },
            ),
        )
        ");

        insta::assert_debug_snapshot!(Size.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List(Default::default()))), &[
            cel_parser::parse("1 + 1").unwrap(), // not an ident
        ])), @r#"
        Err(
            InvalidSyntax {
                message: "takes no arguments",
                syntax: "<this>.size()",
            },
        )
        "#);
    }

    #[test]
    fn test_size_runtime() {
        let registry = ProtoTypeRegistry::new();
        let compiler = Compiler::new(&registry);

        let string_value =
            CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::String)), parse_quote!(input));

        let output = Size
            .compile(CompilerCtx::new(compiler.child(), Some(string_value), &[]))
            .unwrap();

        insta::assert_snapshot!(postcompile::compile_str!(
            postcompile::config! {
                test: true,
                dependencies: vec![
                    postcompile::Dependency::workspace("tinc"),
                ],
            },
            quote::quote! {
                fn size(input: &str) -> Result<u64, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_size() {
                    assert_eq!(size("55").unwrap(), 2);
                }
            },
        ));
    }

    #[test]
    fn test_size_runtime_map() {
        let registry = ProtoTypeRegistry::new();
        let compiler = Compiler::new(&registry);

        let input = CompiledExpr::runtime(
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(
                ProtoValueType::String,
                ProtoValueType::Bool,
            ))),
            parse_quote!(input),
        );

        let output = Size.compile(CompilerCtx::new(compiler.child(), Some(input), &[])).unwrap();

        insta::assert_snapshot!(postcompile::compile_str!(
            postcompile::config! {
                test: true,
                dependencies: vec![
                    postcompile::Dependency::workspace("tinc"),
                ],
            },
            quote::quote! {
                fn size(input: &std::collections::HashMap<String, bool>) -> Result<u64, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_contains() {
                    assert_eq!(size(&{
                        let mut map = std::collections::HashMap::new();
                        map.insert("value".to_string(), true);
                        map
                    }).unwrap(), 1);
                    assert_eq!(size(&std::collections::HashMap::new()).unwrap(), 0);
                    assert_eq!(size(&{
                        let mut map = std::collections::HashMap::new();
                        map.insert("xd".to_string(), true);
                        map.insert("value".to_string(), true);
                        map
                    }).unwrap(), 2);
                }
            },
        ));
    }

    #[test]
    fn test_size_runtime_repeated() {
        let registry = ProtoTypeRegistry::new();
        let compiler = Compiler::new(&registry);

        let string_value = CompiledExpr::runtime(
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(ProtoValueType::String))),
            parse_quote!(input),
        );

        let output = Size
            .compile(CompilerCtx::new(compiler.child(), Some(string_value), &[]))
            .unwrap();

        insta::assert_snapshot!(postcompile::compile_str!(
            postcompile::config! {
                test: true,
                dependencies: vec![
                    postcompile::Dependency::workspace("tinc"),
                ],
            },
            quote::quote! {
                fn size(input: &Vec<String>) -> Result<u64, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_contains() {
                    assert_eq!(size(&vec!["value".into()]).unwrap(), 1);
                    assert_eq!(size(&vec![]).unwrap(), 0);
                    assert_eq!(size(&vec!["xd".into(), "value".into()]).unwrap(), 2);
                }
            },
        ));
    }
}
