use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;

#[derive(Debug, Clone, Default)]
pub(crate) struct Double;

impl Function for Double {
    fn name(&self) -> &'static str {
        "double"
    }

    fn syntax(&self) -> &'static str {
        "<this>.double()"
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
                Ok(CompiledExpr::constant(CelValue::cel_to_double(value)?))
            }
            CompiledExpr::Runtime(RuntimeCompiledExpr { expr, .. }) => Ok(CompiledExpr::runtime(
                CelType::CelValue,
                parse_quote!(::tinc::__private::cel::CelValue::cel_to_double(#expr)?),
            )),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "prost")]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use syn::parse_quote;
    use tinc_cel::CelValue;

    use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerCtx};
    use crate::codegen::cel::functions::{Double, Function};
    use crate::codegen::cel::types::CelType;
    use crate::types::{ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_double_syntax() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(Double.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.double()",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Double.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("13.2".into()))), &[])), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        F64(
                            13.2,
                        ),
                    ),
                },
            ),
        )
        ");

        insta::assert_debug_snapshot!(Double.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List(Default::default()))), &[
            cel_parser::parse("1 + 1").unwrap(), // not an ident
        ])), @r#"
        Err(
            InvalidSyntax {
                message: "takes no arguments",
                syntax: "<this>.double()",
            },
        )
        "#);
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_double_runtime() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let string_value =
            CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::String)), parse_quote!(input));

        let output = Double
            .compile(CompilerCtx::new(compiler.child(), Some(string_value), &[]))
            .unwrap();

        insta::assert_snapshot!(postcompile::compile_str!(
            postcompile::config! {
                test: true,
                dependencies: vec![
                    postcompile::Dependency::version("tinc", "*"),
                ],
            },
            quote::quote! {
                fn to_double(input: &str) -> Result<::tinc::__private::cel::CelValue<'_>, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_to_double() {
                    assert_eq!(to_double("54.4").unwrap(), ::tinc::__private::cel::CelValueConv::conv(54.4));
                }
            },
        ));
    }
}
