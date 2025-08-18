use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub(crate) struct IsNaN;

// this.isNaN(arg) -> arg in this
impl Function for IsNaN {
    fn name(&self) -> &'static str {
        "isNaN"
    }

    fn syntax(&self) -> &'static str {
        "<this>.isNaN()"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = &ctx.this else {
            return Err(CompileError::syntax("missing this", self));
        };

        if !ctx.args.is_empty() {
            return Err(CompileError::syntax("does not take any arguments", self));
        }

        let this = this.clone().into_cel()?;

        match this {
            CompiledExpr::Constant(ConstantCompiledExpr { value }) => {
                Ok(CompiledExpr::constant(CelValue::cel_is_nan(value)?))
            }
            this => Ok(CompiledExpr::runtime(
                CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                parse_quote! {{
                    ::tinc::__private::cel::CelValue::cel_is_nan(
                        #this,
                    )?
                }},
            )),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "prost")]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use syn::parse_quote;
    use tinc_cel::{CelValue, NumberTy};

    use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerCtx};
    use crate::codegen::cel::functions::{Function, IsNaN};
    use crate::codegen::cel::types::CelType;
    use crate::types::{ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_is_nan_syntax() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(IsNaN.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.isNaN()",
            },
        )
        "#);

        insta::assert_debug_snapshot!(IsNaN.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::Number(NumberTy::from(2.0)))), &[])), @r"
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

        insta::assert_debug_snapshot!(IsNaN.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::Number(NumberTy::from(f64::NAN)))), &[])), @r"
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

        insta::assert_debug_snapshot!(IsNaN.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List(Default::default()))), &[
            cel_parser::parse("1 + 1").unwrap(), // not an ident
        ])), @r#"
        Err(
            InvalidSyntax {
                message: "does not take any arguments",
                syntax: "<this>.isNaN()",
            },
        )
        "#);
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_is_nan_runtime() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let double_value =
            CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::Double)), parse_quote!(input));

        let output = IsNaN
            .compile(CompilerCtx::new(compiler.child(), Some(double_value), &[]))
            .unwrap();

        insta::assert_snapshot!(postcompile::compile_str!(
            postcompile::config! {
                test: true,
                dependencies: vec![
                    postcompile::Dependency::path("tinc", ".."),
                ],
            },
            quote::quote! {
                fn is_nan(input: f64) -> Result<bool, ::tinc::__private::cel::CelError<'static>> {
                    Ok(#output)
                }

                #[test]
                fn test_is_nan() {
                    assert_eq!(is_nan(f64::NAN).unwrap(), true);
                    assert_eq!(is_nan(2.0).unwrap(), false);
                }
            },
        ));
    }
}
