use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;

#[derive(Debug, Clone, Default)]
pub(crate) struct Bytes;

impl Function for Bytes {
    fn name(&self) -> &'static str {
        "bytes"
    }

    fn syntax(&self) -> &'static str {
        "<this>.bytes()"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = ctx.this else {
            return Err(CompileError::syntax("missing this", self));
        };

        if !ctx.args.is_empty() {
            return Err(CompileError::syntax("takes no arguments", self));
        }

        match this.into_cel()? {
            CompiledExpr::Constant(ConstantCompiledExpr { value }) => {
                Ok(CompiledExpr::constant(CelValue::cel_to_bytes(value)?))
            }
            CompiledExpr::Runtime(RuntimeCompiledExpr { expr, .. }) => Ok(CompiledExpr::runtime(
                CelType::CelValue,
                parse_quote!(::tinc::__private::cel::CelValue::cel_to_bytes(#expr)?),
            )),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "prost")]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use quote::quote;
    use syn::parse_quote;
    use tinc_cel::CelValue;

    use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerCtx};
    use crate::codegen::cel::functions::{Bytes, Function};
    use crate::codegen::cel::types::CelType;
    use crate::extern_paths::ExternPaths;
    use crate::path_set::PathSet;
    use crate::types::{ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_bytes_syntax() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, ExternPaths::new(crate::Mode::Prost), PathSet::default());
        let compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(Bytes.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.bytes()",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Bytes.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("hi".into()))), &[])), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Bytes(
                        Borrowed(
                            [
                                104,
                                105,
                            ],
                        ),
                    ),
                },
            ),
        )
        ");

        insta::assert_debug_snapshot!(Bytes.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List(Default::default()))), &[
            cel_parser::parse("1 + 1").unwrap(), // not an ident
        ])), @r#"
        Err(
            InvalidSyntax {
                message: "takes no arguments",
                syntax: "<this>.bytes()",
            },
        )
        "#);
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_bytes_runtime() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, ExternPaths::new(crate::Mode::Prost), PathSet::default());
        let compiler = Compiler::new(&registry);

        let string_value =
            CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::String)), parse_quote!(input));

        let output = Bytes
            .compile(CompilerCtx::new(compiler.child(), Some(string_value), &[]))
            .unwrap();

        insta::assert_snapshot!(postcompile::compile_str!(
            postcompile::config! {
                test: true,
                dependencies: vec![
                    postcompile::Dependency::version("tinc", "*"),
                ],
            },
            quote! {
                fn to_bytes(input: &str) -> Result<::tinc::__private::cel::CelValue<'_>, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_to_bytes() {
                    assert_eq!(to_bytes("some string").unwrap(), ::tinc::__private::cel::CelValueConv::conv(b"some string"));
                }
            },
        ));
    }
}
