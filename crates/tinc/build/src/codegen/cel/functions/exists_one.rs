use cel_interpreter::objects::ValueType;
use cel_interpreter::{ExecutionError, FunctionContext};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

#[derive(Debug, Clone, Default)]
pub struct ExistsOne;

// this.existsOne(<ident>, <expr>)
impl Function for ExistsOne {
    fn name(&self) -> &'static str {
        "existsOne"
    }

    fn compile(&self, ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = &ctx.this else {
            return Err(CompileError::MissingTarget {
                func: self.name(),
                message: "this is required when calling the contains function".to_string(),
            });
        };

        if ctx.args.len() != 2 {
            return Err(CompileError::InvalidFunctionArgumentCount {
                func: self.name(),
                expected: 2,
                got: ctx.args.len(),
            });
        }

        let cel_parser::Expression::Ident(variable) = &ctx.args[0] else {
            return Err(CompileError::InvalidFunctionArgument {
                idx: 0,
                message: "variable name as an ident".into(),
                expr: ctx.args[0].clone(),
            });
        };

        let mut child_ctx = ctx.child();

        match &this.ty {
            CelType::CelValue => {
                child_ctx.add_variable(
                    variable,
                    CompiledExpr {
                        expr: parse_quote!(item),
                        ty: CelType::CelValue,
                    },
                );
            }
            CelType::Proto(ProtoType::Modified(
                ProtoModifiedValueType::Repeated(ty) | ProtoModifiedValueType::Map(ty, _),
            )) => {
                child_ctx.add_variable(
                    variable,
                    CompiledExpr {
                        expr: parse_quote!(item),
                        ty: CelType::Proto(ProtoType::Value(ty.clone())),
                    },
                );
            }
            v => return Err(CompileError::FunctionNotFound(format!("no such function map for type {v:?}"))),
        };

        let arg = child_ctx.resolve(&ctx.args[1])?;
        if !arg.ty.can_be_cel() {
            return Err(CompileError::TypeConversion {
                ty: Box::new(arg.ty.clone()),
                message: "the return value of map expr needs to be a CEL value".to_string(),
            });
        }

        let proto_native = |iter: TokenStream| {
            parse_quote! {{
                let mut iter = #iter;
                let mut seen = false;
                loop {
                    let Some(item) = iter.next() else {
                        break seen;
                    };

                    if ::tinc::__private::cel::to_bool(#arg) {
                        if seen {
                            break false;
                        }

                        seen = true;
                    }
                }
            }}
        };

        Ok(CompiledExpr {
            expr: match &this.ty {
                CelType::CelValue => parse_quote! {
                    ::tinc::__private::cel::CelValue::cel_exists_one(#this, |item| {
                        ::core::result::Result::Ok(
                            ::tinc::__private::cel::to_bool(#arg)
                        )
                    })
                },
                CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(_, _))) => {
                    proto_native(quote!((#this).keys()))
                }
                CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(_))) => {
                    proto_native(quote!((#this).iter()))
                }
                _ => unreachable!(),
            },
            ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
        })
    }

    fn interpret(&self, fctx: &FunctionContext) -> Result<cel_interpreter::Value, ExecutionError> {
        let Some(this) = &fctx.this else {
            return Err(ExecutionError::missing_argument_or_target());
        };

        if fctx.args.len() != 2 {
            return Err(ExecutionError::invalid_argument_count(1, fctx.args.len()));
        }

        let cel_parser::Expression::Ident(variable) = &fctx.args[0] else {
            return Err(ExecutionError::FunctionError {
                function: self.name().to_owned(),
                message: "variable name as an ident".to_owned(),
            });
        };

        fn handle(
            mut i: impl Iterator<Item = cel_interpreter::Value>,
            fctx: &FunctionContext,
            variable: &str,
        ) -> Result<bool, ExecutionError> {
            let mut seen = false;
            loop {
                let Some(item) = i.next() else {
                    break Ok(seen);
                };

                let mut ctx = fctx.ptx.new_inner_scope();
                ctx.add_variable_from_value(variable, item);
                let item = ctx.resolve(&fctx.args[1])?;

                if matches!(item, cel_interpreter::Value::Bool(true)) {
                    if seen {
                        break Ok(false);
                    }

                    seen = true;
                }
            }
        }

        match this {
            cel_interpreter::Value::List(s) => Ok(cel_interpreter::Value::Bool(handle(s.iter().cloned(), fctx, variable)?)),
            cel_interpreter::Value::Map(map) => Ok(cel_interpreter::Value::Bool(handle(
                map.map.keys().cloned().map(Into::into),
                fctx,
                variable,
            )?)),
            item => Err(item.error_expected_type(ValueType::List)),
        }
    }
}
