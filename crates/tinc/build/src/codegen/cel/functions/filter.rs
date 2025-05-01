use cel_interpreter::objects::ValueType;
use cel_interpreter::{ExecutionError, FunctionContext};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

use super::Function;
use crate::codegen::cel::compiler::{CompileError, CompiledExpr, CompilerCtx};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType};

pub struct Filter;

// this.filter(<ident>, <expr>)
impl Function for Filter {
    const NAME: &'static str = "filter";

    fn compile(ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = &ctx.this else {
            return Err(CompileError::MissingTarget {
                func: Self::NAME,
                message: "this is required when calling the contains function".to_string(),
            });
        };

        if ctx.args.len() != 2 {
            return Err(CompileError::InvalidFunctionArgumentCount {
                func: Self::NAME,
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
                let mut items = Vec::new();
                for item in #iter {
                    if ::tinc::__private::cel::to_bool(#arg) {
                        items.push(::tinc::__private::cel::CelValueConv::conv(item));
                    }
                }

                ::tinc::__private::cel::CelValue::List(items.into())
            }}
        };

        Ok(CompiledExpr {
            expr: match &this.ty {
                CelType::CelValue => parse_quote! {
                    ::tinc::__private::cel::CelValue::cel_filter(#this, |item| {
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
            ty: CelType::CelValue,
        })
    }

    fn interpret(fctx: &FunctionContext) -> Result<cel_interpreter::Value, ExecutionError> {
        let Some(this) = &fctx.this else {
            return Err(ExecutionError::missing_argument_or_target());
        };

        if fctx.args.len() != 2 {
            return Err(ExecutionError::invalid_argument_count(1, fctx.args.len()));
        }

        let cel_parser::Expression::Ident(variable) = &fctx.args[0] else {
            return Err(ExecutionError::FunctionError {
                function: Self::NAME.to_owned(),
                message: "variable name as an ident".to_owned(),
            });
        };

        fn handle(
            i: impl Iterator<Item = cel_interpreter::Value>,
            fctx: &FunctionContext,
            variable: &str,
        ) -> Result<cel_interpreter::Value, ExecutionError> {
            Ok(cel_interpreter::Value::List(
                i.filter_map(|value| {
                    let mut child = fctx.ptx.new_inner_scope();
                    child.add_variable_from_value(variable, value.clone());
                    match child.resolve(&fctx.args[0]) {
                        Err(err) => Some(Err(err)),
                        Ok(cel_interpreter::Value::Bool(true)) => Some(Ok(value.clone())),
                        Ok(_) => None,
                    }
                })
                .collect::<Result<Vec<_>, _>>()?
                .into(),
            ))
        }

        match this {
            cel_interpreter::Value::List(s) => handle(s.iter().cloned(), fctx, variable),
            cel_interpreter::Value::Map(map) => handle(map.map.keys().cloned().map(Into::into), fctx, variable),
            item => Err(item.error_expected_type(ValueType::List)),
        }
    }
}
