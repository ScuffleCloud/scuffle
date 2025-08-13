use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoPath, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub(crate) struct Enum(pub Option<ProtoPath>);

impl Function for Enum {
    fn name(&self) -> &'static str {
        "enum"
    }

    fn syntax(&self) -> &'static str {
        "<this>.enum() | <this>.enum(<path>)"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = ctx.this.as_ref() else {
            return Err(CompileError::syntax("missing this", self));
        };

        if ctx.args.len() > 1 {
            return Err(CompileError::syntax("invalid number of arguments", self));
        }

        let enum_path = if let Some(arg) = ctx.args.first() {
            ctx.resolve(arg)?
        } else {
            match (&this, &self.0) {
                (
                    CompiledExpr::Runtime(RuntimeCompiledExpr {
                        ty:
                            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Enum(path))))
                            | CelType::Proto(ProtoType::Value(ProtoValueType::Enum(path))),
                        ..
                    }),
                    _,
                )
                | (_, Some(path)) => CompiledExpr::Constant(ConstantCompiledExpr {
                    value: CelValue::String(path.0.clone().into()),
                }),
                _ => {
                    return Err(CompileError::syntax(
                        "unable to determine enum type, try providing an explicit path",
                        self,
                    ));
                }
            }
        };

        let this = this.clone().into_cel()?;
        let enum_path = enum_path.into_cel()?;

        match (this, enum_path) {
            (
                CompiledExpr::Constant(ConstantCompiledExpr { value: this }),
                CompiledExpr::Constant(ConstantCompiledExpr { value: enum_path }),
            ) => Ok(CompiledExpr::constant(CelValue::cel_to_enum(this, enum_path)?)),
            (this, enum_path) => Ok(CompiledExpr::runtime(
                CelType::CelValue,
                parse_quote! {
                    ::tinc::__private::cel::CelValue::cel_to_enum(
                        #this,
                        #enum_path,
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

    use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerCtx};
    use crate::codegen::cel::functions::{Enum, Function};
    use crate::codegen::cel::types::CelType;
    use crate::types::{ProtoType, ProtoTypeRegistry, ProtoValueType};

    #[test]
    fn test_enum_syntax() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);
        let enum_ = Enum(None);
        insta::assert_debug_snapshot!(enum_.compile(CompilerCtx::new(compiler.child(), None, &[])), @r#"
        Err(
            InvalidSyntax {
                message: "missing this",
                syntax: "<this>.enum() | <this>.enum(<path>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(enum_.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(5)), &[])), @r#"
        Err(
            InvalidSyntax {
                message: "unable to determine enum type, try providing an explicit path",
                syntax: "<this>.enum() | <this>.enum(<path>)",
            },
        )
        "#);

        insta::assert_debug_snapshot!(enum_.compile(CompilerCtx::new(compiler.child(), Some(CompiledExpr::constant(5)), &[
            cel_parser::parse("'some.Enum'").unwrap(),
        ])), @r#"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Enum(
                        CelEnum {
                            tag: Owned(
                                "some.Enum",
                            ),
                            value: 5,
                        },
                    ),
                },
            ),
        )
        "#);
    }

    #[test]
    #[cfg(not(valgrind))]
    fn test_enum_runtime() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let string_value =
            CompiledExpr::runtime(CelType::Proto(ProtoType::Value(ProtoValueType::Int32)), parse_quote!(input));

        let output = Enum(None)
            .compile(CompilerCtx::new(
                compiler.child(),
                Some(string_value),
                &[cel_parser::parse("'some.Enum'").unwrap()],
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
                fn to_enum(input: i32) -> Result<::tinc::__private::cel::CelValue<'static>, ::tinc::__private::cel::CelError<'static>> {
                    Ok(#output)
                }

                #[test]
                fn test_to_enum() {
                    #[::tinc::reexports::linkme::distributed_slice(::tinc::__private::cel::TINC_CEL_ENUM_VTABLE)]
                    #[linkme(crate = ::tinc::reexports::linkme)]
                    static ENUM_VTABLE: ::tinc::__private::cel::EnumVtable = ::tinc::__private::cel::EnumVtable {
                        proto_path: "some.Enum",
                        is_valid: |_| {
                            true
                        },
                        to_serde: |_| {
                            ::tinc::__private::cel::CelValue::String(::tinc::__private::cel::CelString::Borrowed("SERDE"))
                        },
                        to_proto: |_| {
                            ::tinc::__private::cel::CelValue::String(::tinc::__private::cel::CelString::Borrowed("PROTO"))
                        }
                    };

                    ::tinc::__private::cel::CelMode::Serde.set();
                    assert_eq!(to_enum(1).unwrap().to_string(), "SERDE");
                    ::tinc::__private::cel::CelMode::Proto.set();
                    assert_eq!(to_enum(1).unwrap().to_string(), "PROTO");
                }
            },
        ));
    }
}
