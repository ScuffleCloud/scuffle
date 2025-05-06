use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;

#[derive(Debug, Clone, Default)]
pub(crate) struct Int;

impl Function for Int {
    fn name(&self) -> &'static str {
        "int"
    }

    fn syntax(&self) -> &'static str {
        "<this>.int()"
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
                Ok(CompiledExpr::constant(CelValue::cel_to_int(value)?))
            }
            CompiledExpr::Runtime(RuntimeCompiledExpr { expr, .. }) => Ok(CompiledExpr::runtime(
                CelType::CelValue,
                parse_quote!(::tinc::__private::cel::CelValue::cel_to_int(#expr)?),
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
    use crate::codegen::cel::functions::{Function, Int};
    use crate::codegen::cel::types::CelType;
    use crate::types::{ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_int_syntax() {
        let registry = ProtoTypeRegistry::new();
        let compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(Int.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.int()",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Int.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("13".into()))), &[])), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            13,
                        ),
                    ),
                },
            ),
        )
        ");

        insta::assert_debug_snapshot!(Int.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List(Default::default()))), &[
            cel_parser::parse("1 + 1").unwrap(), // not an ident
        ])), @r#"
        Err(
            InvalidSyntax {
                message: "takes no arguments",
                syntax: "<this>.int()",
            },
        )
        "#);
    }

    #[test]
    fn test_int_runtime() {
        let registry = ProtoTypeRegistry::new();
        let compiler = Compiler::new(&registry);

        let string_value =
            CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::String)), parse_quote!(input));

        let output = Int
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
                fn to_int(input: &str) -> Result<::tinc::__private::cel::CelValue<'_>, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_to_int() {
                    assert_eq!(to_int("55").unwrap(), ::tinc::__private::cel::CelValueConv::conv(55));
                }
            },
        ));
    }
}
