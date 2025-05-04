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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use quote::quote;
    use syn::parse_quote;
    use tinc_cel::CelValue;

    use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerCtx};
    use crate::codegen::cel::functions::{Double, Function};
    use crate::codegen::cel::types::CelType;
    use crate::types::{ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_bytes_syntax() {
        let registry = ProtoTypeRegistry::new();
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
    fn test_double_runtime() {
        let registry = ProtoTypeRegistry::new();
        let compiler = Compiler::new(&registry);

        let string_value =
            CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::String)), parse_quote!(input));

        let result = Double
            .compile(CompilerCtx::new(compiler.child(), Some(string_value), &[]))
            .unwrap();

        let small_fn = quote! {
            #[allow(dead_code)]
            fn double_conv(input: &std::string::String) -> Result<::tinc::__private::cel::CelValue<'_>, ::tinc::__private::cel::CelError<'_>> {
                Ok(#result)
            }
        };

        let compiled = postcompile::compile_str!(&small_fn.to_string());
        insta::assert_snapshot!(compiled);
    }
}
