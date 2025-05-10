use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub(crate) struct Matches;

// this.matches(arg) -> arg in this
impl Function for Matches {
    fn name(&self) -> &'static str {
        "matches"
    }

    fn syntax(&self) -> &'static str {
        "<this>.matches(<const regex>)"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = &ctx.this else {
            return Err(CompileError::syntax("missing this", self));
        };

        if ctx.args.len() != 1 {
            return Err(CompileError::syntax("takes exactly one argument", self));
        }

        let CompiledExpr::Constant(ConstantCompiledExpr {
            value: CelValue::String(regex),
        }) = ctx.resolve(&ctx.args[0])?.into_cel()?
        else {
            return Err(CompileError::syntax("regex must be known at compile time string", self));
        };

        let regex = regex.as_ref();
        if regex.is_empty() {
            return Err(CompileError::syntax("regex cannot be an empty string", self));
        }

        let re = regex::Regex::new(regex).map_err(|err| CompileError::syntax(format!("bad regex {err}"), self))?;

        let this = this.clone().into_cel()?;

        match this {
            CompiledExpr::Constant(ConstantCompiledExpr { value }) => {
                Ok(CompiledExpr::constant(CelValue::cel_matches(value, &re)?))
            }
            this => Ok(CompiledExpr::runtime(
                CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                parse_quote! {{
                    static REGEX: ::std::sync::LazyLock<::tinc::reexports::regex::Regex> = ::std::sync::LazyLock::new(|| {
                        ::tinc::reexports::regex::Regex::new(#regex).expect("failed to compile regex this is a bug in tinc")
                    });

                    ::tinc::__private::cel::CelValue::cel_matches(
                        #this,
                        &*REGEX,
                    )?
                }},
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
    use crate::codegen::cel::functions::{Function, Matches};
    use crate::codegen::cel::types::CelType;
    use crate::types::{ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_matches_syntax() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(Matches.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.matches(<const regex>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Matches.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("hi".into()))), &[])), @r#"
        Err(
            InvalidSyntax {
                message: "takes exactly one argument",
                syntax: "<this>.matches(<const regex>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Matches.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("hi".into()))), &[
            cel_parser::parse("dyn('^h')").unwrap(),
        ])), @r#"
        Err(
            InvalidSyntax {
                message: "regex must be known at compile time string",
                syntax: "<this>.matches(<const regex>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Matches.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("hi".into()))), &[
            cel_parser::parse("'^h'").unwrap(),
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
    fn test_matches_runtime_string() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let string_value =
            CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::String)), parse_quote!(input));

        let output = Matches
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[cel_parser::parse("'\\\\d+'").unwrap()],
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
                fn matches(input: &String) -> Result<bool, ::tinc::__private::cel::CelError<'_>> {
                    Ok(#output)
                }

                #[test]
                fn test_matches() {
                    assert_eq!(matches(&"in2dastring".into()).unwrap(), true);
                    assert_eq!(matches(&"in3dastring".into()).unwrap(), true);
                    assert_eq!(matches(&"xd".into()).unwrap(), false);
                }
            },
        ));
    }
}
