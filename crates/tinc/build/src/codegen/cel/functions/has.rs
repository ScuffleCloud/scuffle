use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};

#[derive(Debug, Clone, Default)]
pub(crate) struct Has;

// has(field-arg)
impl Function for Has {
    fn name(&self) -> &'static str {
        "has"
    }

    fn syntax(&self) -> &'static str {
        "has(<field accessor>)"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        if ctx.this.is_some() {
            return Err(CompileError::syntax("function has no this", self));
        };

        if ctx.args.len() != 1 {
            return Err(CompileError::syntax("invalid arguments", self));
        }

        let arg = ctx.resolve(&ctx.args[0]);

        Ok(CompiledExpr::constant(arg.is_ok()))
    }
}

#[cfg(test)]
#[cfg(feature = "prost")]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use tinc_cel::CelValue;

    use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerCtx};
    use crate::codegen::cel::functions::{Function, Has};
    use crate::types::ProtoTypeRegistry;

    #[test]
    fn test_has_syntax() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let mut compiler = Compiler::new(&registry);
        insta::assert_debug_snapshot!(Has.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "invalid arguments",
                syntax: "has(<field accessor>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Has.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(CelValue::String("hi".into()))), &[])), @r#"
        Err(
            InvalidSyntax {
                message: "function has no this",
                syntax: "has(<field accessor>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(Has.compile(CompilerCtx::new(compiler.child(), None, &[
            cel_parser::parse("x").unwrap(),
        ])), @r"
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

        compiler.add_variable("x", CompiledExpr::constant(CelValue::Null));

        insta::assert_debug_snapshot!(Has.compile(CompilerCtx::new(compiler.child(), None, &[
            cel_parser::parse("x").unwrap(),
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
}
