use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub(crate) struct IsIpv6;

// this.isIpv6(arg) -> arg in this
impl Function for IsIpv6 {
    fn name(&self) -> &'static str {
        "isIpv6"
    }

    fn syntax(&self) -> &'static str {
        "<this>.isIpv6()"
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
                Ok(CompiledExpr::constant(CelValue::cel_is_ipv6(value)?))
            }
            this => Ok(CompiledExpr::runtime(
                CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                parse_quote! {{
                    ::tinc::__private::cel::CelValue::cel_is_ipv6(
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
    use crate::codegen::cel::functions::{Function, IsIpv6};
    use crate::codegen::cel::types::CelType;
    use crate::extern_paths::ExternPaths;
    use crate::path_set::PathSet;
    use crate::types::{ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_is_ipv6_syntax() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, ExternPaths::new(crate::Mode::Prost), PathSet::default());
        let compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(IsIpv6.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.isIpv6()",
            },
        )
        "#);

        insta::assert_debug_snapshot!(IsIpv6.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("192.168.1.1".into()))), &[])), @r"
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

        insta::assert_debug_snapshot!(IsIpv6.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("fe80::4c1a:18ff:fe0b:7946".into()))), &[])), @r"
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

        insta::assert_debug_snapshot!(IsIpv6.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::List(Default::default()))), &[
            cel_parser::parse("1 + 1").unwrap(), // not an ident
        ])), @r#"
        Err(
            InvalidSyntax {
                message: "does not take any arguments",
                syntax: "<this>.isIpv6()",
            },
        )
        "#);
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_is_ipv6_runtime() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, ExternPaths::new(crate::Mode::Prost), PathSet::default());
        let compiler = Compiler::new(&registry);

        let string_value =
            CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::String)), parse_quote!(input));

        let output = IsIpv6
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
                fn is_ipv6(input: &str) -> Result<bool, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_is_hostname() {
                    assert_eq!(is_ipv6("fe80::4c1a:18ff:fe0b:7946").unwrap(), true);
                    assert_eq!(is_ipv6("192.168.1.1").unwrap(), false);
                }
            },
        ));
    }
}
