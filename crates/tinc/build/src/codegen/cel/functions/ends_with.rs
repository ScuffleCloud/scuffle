use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub(crate) struct EndsWith;

// this.endsWith(arg) -> arg in this
impl Function for EndsWith {
    fn name(&self) -> &'static str {
        "endsWith"
    }

    fn syntax(&self) -> &'static str {
        "<this>.endsWith(<arg>)"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = &ctx.this else {
            return Err(CompileError::syntax("missing this", self));
        };

        if ctx.args.len() != 1 {
            return Err(CompileError::syntax("takes exactly one argument", self));
        }

        let arg = ctx.resolve(&ctx.args[0])?.into_cel()?;
        let this = this.clone().into_cel()?;

        match (this, arg) {
            (
                CompiledExpr::Constant(ConstantCompiledExpr { value: this }),
                CompiledExpr::Constant(ConstantCompiledExpr { value: arg }),
            ) => Ok(CompiledExpr::constant(CelValue::cel_ends_with(this, arg)?)),
            (this, arg) => Ok(CompiledExpr::runtime(
                CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                parse_quote! {
                    ::tinc::__private::cel::CelValue::cel_ends_with(
                        #this,
                        #arg,
                    )?
                },
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
    use crate::codegen::cel::functions::{EndsWith, Function};
    use crate::codegen::cel::types::CelType;
    use crate::types::{ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_ends_with_syntax() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(EndsWith.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.endsWith(<arg>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(EndsWith.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("13.2".into()))), &[])), @r#"
        Err(
            InvalidSyntax {
                message: "takes exactly one argument",
                syntax: "<this>.endsWith(<arg>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(EndsWith.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant("some string")), &[
            cel_parser::parse("'ing'").unwrap(), // not an ident
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
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_ends_with_runtime() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let string_value =
            CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::String)), parse_quote!(input));

        let output = EndsWith
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[
                    cel_parser::parse("'ing'").unwrap(), // not an ident
                ],
            ))
            .unwrap();

        insta::assert_snapshot!(postcompile::compile_str!(
            postcompile::config! {
                test: true,
                dependencies: vec![
                    postcompile::Dependency::version("tinc", "*"),
                ],
            },
            quote::quote! {
                fn ends_with(input: &str) -> Result<bool, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_to_double() {
                    assert_eq!(ends_with("testing").unwrap(), true);
                    assert_eq!(ends_with("smile").unwrap(), false);
                }
            },
        ));
    }
}
