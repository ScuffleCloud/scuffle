use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub(crate) struct IsUlid;

// this.isUlid(arg) -> arg in this
impl Function for IsUlid {
    fn name(&self) -> &'static str {
        "isUlid"
    }

    fn syntax(&self) -> &'static str {
        "<this>.isUlid()"
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
                Ok(CompiledExpr::constant(CelValue::cel_is_ulid(value)?))
            }
            this => Ok(CompiledExpr::runtime(
                CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                parse_quote! {{
                    ::tinc::__private::cel::CelValue::cel_is_ulid(
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
    use tinc_cel::CelValue;

    use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerCtx};
    use crate::codegen::cel::functions::{Function, IsUlid};
    use crate::codegen::cel::types::CelType;
    use crate::types::{ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_is_ulid_syntax() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(IsUlid.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.isUlid()",
            },
        )
        "#);

        insta::assert_debug_snapshot!(IsUlid.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("xdd".into()))), &[])), @r"
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

        insta::assert_debug_snapshot!(IsUlid.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("01F6MKTFTG0009C9ZSNZTFV2ZF".into()))), &[])), @r"
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

        insta::assert_debug_snapshot!(IsUlid.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List(Default::default()))), &[
            cel_parser::parse("1 + 1").unwrap(), // not an ident
        ])), @r#"
        Err(
            InvalidSyntax {
                message: "does not take any arguments",
                syntax: "<this>.isUlid()",
            },
        )
        "#);
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_is_ulid_runtime() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let string_value =
            CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::String)), parse_quote!(input));

        let output = IsUlid
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
                fn is_ulid(input: &str) -> Result<bool, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_is_ulid() {
                    assert_eq!(is_ulid("01F6MKTFTG0009C9ZSNZTFV2ZF").unwrap(), true);
                    assert_eq!(is_ulid("not-a-ulid").unwrap(), false);
                }
            },
        ));
    }
}
