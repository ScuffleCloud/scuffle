use cel_parser::{ArithmeticOp, Atom, Expression, Member, RelationOp};
use quote::quote;
use syn::parse_quote;

use super::{CompileError, CompiledExpr, Compiler, CompilerCtx, helpers};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

pub fn resolve(ctx: &Compiler, expr: &Expression) -> Result<CompiledExpr, CompileError> {
    match expr {
        Expression::And(left, right) => resolve_and(ctx, left, right),
        Expression::Arithmetic(left, op, right) => resolve_arithmetic(ctx, left, op, right),
        Expression::Atom(atom) => resolve_atom(ctx, atom),
        Expression::FunctionCall(func, this, args) => resolve_function_call(ctx, func, this.as_deref(), args),
        Expression::Ident(ident) => resolve_ident(ctx, ident),
        Expression::List(items) => resolve_list(ctx, items),
        Expression::Map(items) => resolve_map(ctx, items),
        Expression::Member(expr, member) => resolve_member(ctx, expr, member),
        Expression::Or(left, right) => resolve_or(ctx, left, right),
        Expression::Relation(left, op, right) => resolve_relation(ctx, left, op, right),
        Expression::Ternary(cond, left, right) => resolve_ternary(ctx, cond, left, right),
        Expression::Unary(op, expr) => resolve_unary(ctx, op, expr),
    }
}

fn resolve_and(ctx: &Compiler, left: &Expression, right: &Expression) -> Result<CompiledExpr, CompileError> {
    let left = ctx.resolve(left)?;
    let right = ctx.resolve(right)?;
    let left = helpers::to_bool(left);
    let right = helpers::to_bool(right);
    Ok(CompiledExpr {
        expr: parse_quote! {
            (#left) && (#right)
        },
        ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
    })
}

fn resolve_arithmetic(
    ctx: &Compiler,
    left: &Expression,
    op: &ArithmeticOp,
    right: &Expression,
) -> Result<CompiledExpr, CompileError> {
    let left = ctx.resolve(left)?;
    if !left.ty.can_be_cel() {
        return Err(CompileError::type_conversion(
            left.ty,
            "cannot perform arithmetic operation on a non-CEL value",
        ));
    }
    let right = ctx.resolve(right)?;
    if !right.ty.can_be_cel() {
        return Err(CompileError::type_conversion(
            right.ty,
            "cannot perform arithmetic operation on a non-CEL value",
        ));
    }

    let op = match op {
        ArithmeticOp::Add => quote! { cel_add },
        ArithmeticOp::Subtract => quote! { cel_sub },
        ArithmeticOp::Divide => quote! { cel_div },
        ArithmeticOp::Multiply => quote! { cel_mul },
        ArithmeticOp::Modulus => quote! { cel_rem },
    };

    Ok(CompiledExpr {
        expr: parse_quote! {
            ::tinc::__private::cel::CelValue::#op(
                #right,
                #left,
            )?
        },
        ty: CelType::CelValue,
    })
}

fn resolve_atom(_: &Compiler, atom: &Atom) -> Result<CompiledExpr, CompileError> {
    match atom {
        Atom::Int(i) => Ok(CompiledExpr {
            expr: parse_quote! {
                ::tinc::__private::cel::CelValue::Number(::tinc::__private::cel::NumberTy::I64(#i))
            },
            ty: CelType::CelValue,
        }),
        Atom::UInt(i) => Ok(CompiledExpr {
            expr: parse_quote! {
                ::tinc::__private::cel::CelValue::Number(::tinc::__private::cel::NumberTy::U64(#i))
            },
            ty: CelType::CelValue,
        }),
        Atom::Float(f) => Ok(CompiledExpr {
            expr: parse_quote! {
                ::tinc::__private::cel::CelValue::Number(::tinc::__private::cel::NumberTy::F64(#f))
            },
            ty: CelType::CelValue,
        }),
        Atom::String(s) => {
            let s = s.as_str();
            Ok(CompiledExpr {
                expr: parse_quote! {
                    ::tinc::__private::cel::CelValue::StringRef(#s)
                },
                ty: CelType::CelValue,
            })
        }
        Atom::Bytes(b) => {
            let b = syn::LitByteStr::new(b, proc_macro2::Span::call_site());
            Ok(CompiledExpr {
                expr: parse_quote! {
                    ::tinc::__private::cel::CelValue::BytesRef(#b)
                },
                ty: CelType::CelValue,
            })
        }
        Atom::Bool(b) => Ok(CompiledExpr {
            expr: parse_quote! {
                ::tinc::__private::cel::CelValue::Bool(#b)
            },
            ty: CelType::CelValue,
        }),
        Atom::Null => Ok(CompiledExpr {
            expr: parse_quote! {
                ::tinc::__private::cel::CelValue::Null
            },
            ty: CelType::CelValue,
        }),
    }
}

fn resolve_function_call(
    ctx: &Compiler,
    func: &Expression,
    this: Option<&Expression>,
    args: &[Expression],
) -> Result<CompiledExpr, CompileError> {
    let Expression::Ident(func_name) = func else {
        return Err(CompileError::UnsupportedFunctionCallIdentifierType(func.clone()));
    };

    let Some(func) = ctx.get_function(func_name) else {
        return Err(CompileError::FunctionNotFound(func_name.to_string()));
    };

    let this = if let Some(this) = this {
        Some(ctx.resolve(this)?)
    } else {
        None
    };

    (func.compile)(CompilerCtx::new(ctx.child(), this, args))
}

fn resolve_ident(ctx: &Compiler, ident: &str) -> Result<CompiledExpr, CompileError> {
    ctx.get_variable(ident)
        .cloned()
        .ok_or_else(|| CompileError::VariableNotFound(ident.to_owned()))
}

fn resolve_list(ctx: &Compiler, items: &[Expression]) -> Result<CompiledExpr, CompileError> {
    let items = items.iter().map(|item| ctx.resolve(item)).collect::<Result<Vec<_>, _>>()?;

    if let Some(item) = items.iter().find(|item| !item.ty.can_be_cel()) {
        return Err(CompileError::TypeConversion {
            ty: Box::new(item.ty.clone()),
            message: "can only contain items that can be converted to CEL".to_string(),
        });
    }

    Ok(CompiledExpr {
        expr: parse_quote! {
            ::tinc::__private::cel::CelValueConv::List(::std::iter::FromIterator::from_iter([
                #(
                    ::tinc::__private::cel::CelValueConv::conv(#items)
                ),*
            ]))
        },
        ty: CelType::CelValue,
    })
}

fn resolve_map(ctx: &Compiler, items: &[(Expression, Expression)]) -> Result<CompiledExpr, CompileError> {
    dbg!(items);

    let items = items
        .iter()
        .map(|(key, value)| {
            let key = ctx.resolve(key)?;
            let value = ctx.resolve(value)?;
            if !key.ty.can_be_cel() {
                return Err(CompileError::TypeConversion {
                    ty: Box::new(key.ty.clone()),
                    message: "can only contain keys that can be converted to CEL".to_string(),
                });
            }
            if !value.ty.can_be_cel() {
                return Err(CompileError::TypeConversion {
                    ty: Box::new(value.ty.clone()),
                    message: "can only contain values that can be converted to CEL".to_string(),
                });
            }
            Ok(quote! {
                (
                    ::tinc::__private::cel::CelValueConv::conv(#key),
                    ::tinc::__private::cel::CelValueConv::conv(#value),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CompiledExpr {
        expr: parse_quote! {
            ::tinc::__private::cel::CelValueConv::Map(::std::iter::FromIterator::from_iter([
                #(#items),*
            ]))
        },
        ty: CelType::CelValue,
    })
}

fn resolve_member(ctx: &Compiler, expr: &Expression, member: &Member) -> Result<CompiledExpr, CompileError> {
    let expr = ctx.resolve(expr)?;
    match member {
        Member::Attribute(attr) => {
            let attr = attr.as_str();
            match &expr.ty {
                CelType::CelValue => Ok(CompiledExpr {
                    expr: parse_quote! {
                        (#expr).access(#attr)?
                    },
                    ty: CelType::CelValue,
                }),
                CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Message(
                    full_name,
                )))) => {
                    let msg = ctx
                        .registry()
                        .get_message(full_name)
                        .ok_or_else(|| CompileError::MissingMessage(full_name.clone()))?;

                    let field_ty = msg.fields.get(attr).ok_or_else(|| CompileError::MemberAccess {
                        ty: Box::new(expr.ty.clone()),
                        message: format!("message {} does not have field {}", msg.full_name, attr),
                    })?;

                    let field_ident = field_ty.rust_ident();

                    Ok(CompiledExpr {
                        ty: CelType::Proto(field_ty.ty.clone()),
                        expr: parse_quote! {
                            match (#expr) {
                                Some(value) => &value.#field_ident,
                                None => return Err(::tinc::__private::cel::CelError::BadAccess {
                                    member: ::tinc::__private::cel::CelValue::StringRef(#attr),
                                    container: ::tinc::__private::cel::CelValue::Null,
                                }),
                            }
                        },
                    })
                }
                CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::OneOf(oneof))) => {
                    let field_ty = oneof.fields.get(attr).ok_or_else(|| CompileError::MemberAccess {
                        ty: Box::new(expr.ty.clone()),
                        message: format!("oneof {} does not have field {}", oneof.full_name, attr),
                    })?;

                    let field_ident = field_ty.rust_ident();

                    Ok(CompiledExpr {
                        ty: CelType::Proto(ProtoType::Value(field_ty.ty.clone())),
                        expr: parse_quote! {
                            match (#expr) {
                                Some(value) => &value.#field_ident,
                                None => return Err(::tinc::__private::cel::CelError::BadAccess {
                                    member: ::tinc::__private::cel::CelValue::StringRef(#attr),
                                    container: ::tinc::__private::cel::CelValue::Null,
                                }),
                            }
                        },
                    })
                }
                CelType::Proto(ProtoType::Value(ProtoValueType::Message(full_name))) => {
                    let msg = ctx
                        .registry()
                        .get_message(full_name)
                        .ok_or_else(|| CompileError::MissingMessage(full_name.clone()))?;
                    let field_ty = msg.fields.get(attr).ok_or_else(|| CompileError::MemberAccess {
                        ty: Box::new(expr.ty.clone()),
                        message: format!("message {} does not have field {}", msg.full_name, attr),
                    })?;

                    let field_ident = field_ty.rust_ident();

                    Ok(CompiledExpr {
                        ty: CelType::Proto(field_ty.ty.clone()),
                        expr: parse_quote! {
                            &(#expr).#field_ident,
                        },
                    })
                }
                CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(ProtoValueType::String, value_ty))) => {
                    Ok(CompiledExpr {
                        ty: CelType::Proto(ProtoType::Value(value_ty.clone())),
                        expr: parse_quote! {
                            ::tinc::__private::cel::CelValueConv::map_access(
                                #expr,
                                #attr,
                            )?
                        },
                    })
                }
                _ => Err(CompileError::MemberAccess {
                    ty: Box::new(expr.ty.clone()),
                    message: "can only access attributes on messages and maps with string keys".to_string(),
                }),
            }
        }
        Member::Index(idx) => {
            let idx = ctx.resolve(idx)?;
            match &expr.ty {
                CelType::CelValue => Ok(CompiledExpr {
                    expr: parse_quote! {},
                    ty: CelType::CelValue,
                }),
                CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(item_ty))) => Ok(CompiledExpr {
                    ty: CelType::Proto(ProtoType::Value(item_ty.clone())),
                    expr: parse_quote! {
                        ::tinc::__private::cel::CelValueConv::array_access(
                            #expr,
                            #idx,
                        )?
                    },
                }),
                CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(_, value_ty))) => Ok(CompiledExpr {
                    ty: CelType::Proto(ProtoType::Value(value_ty.clone())),
                    expr: parse_quote! {
                        ::tinc::__private::cel::CelValueConv::map_access(
                            #expr,
                            #idx,
                        )?
                    },
                }),
                _ => Err(CompileError::MemberAccess {
                    ty: Box::new(expr.ty.clone()),
                    message: "cannot index into non-repeated and non-map values".to_string(),
                }),
            }
        }
        Member::Fields(_) => Err(CompileError::NotImplemented),
    }
}

fn resolve_or(ctx: &Compiler, left: &Expression, right: &Expression) -> Result<CompiledExpr, CompileError> {
    let left = ctx.resolve(left)?;
    let right = ctx.resolve(right)?;
    let left = helpers::to_bool(left);
    let right = helpers::to_bool(right);
    Ok(CompiledExpr {
        expr: parse_quote! {
            (#left) || (#right)
        },
        ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
    })
}

fn resolve_relation(
    ctx: &Compiler,
    left: &Expression,
    op: &RelationOp,
    right: &Expression,
) -> Result<CompiledExpr, CompileError> {
    let left = ctx.resolve(left)?;
    if !left.ty.can_be_cel() {
        return Err(CompileError::TypeConversion {
            ty: Box::new(left.ty.clone()),
            message: format!("cannot perform relational operation {op:?} on a non-CEL value"),
        });
    }

    let right = ctx.resolve(right)?;
    if let (
        RelationOp::In,
        CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(item) | ProtoModifiedValueType::Map(item, _))),
    ) = (op, &right.ty)
    {
        if !matches!(item, ProtoValueType::Message { .. } | ProtoValueType::Enum(_)) {
            let op = match &right.ty {
                CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(_))) => {
                    quote! { array_contains }
                }
                CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(_, _))) => quote! { map_contains },
                _ => unreachable!(),
            };

            return Ok(CompiledExpr {
                expr: parse_quote! {
                    ::tinc::__private::cel::#op(
                        #right,
                        #left,
                    )
                },
                ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
            });
        }
    }

    if !right.ty.can_be_cel() {
        return Err(CompileError::TypeConversion {
            ty: Box::new(right.ty.clone()),
            message: format!("cannot perform relational operation {op:?} on a non-CEL value"),
        });
    }

    let op = match op {
        RelationOp::LessThan => quote! { cel_lt },
        RelationOp::LessThanEq => quote! { cel_lte },
        RelationOp::GreaterThan => quote! { cel_gt },
        RelationOp::GreaterThanEq => quote! { cel_gte },
        RelationOp::Equals => quote! { cel_eq },
        RelationOp::NotEquals => quote! { cel_neq },
        RelationOp::In => quote! { cel_contained_by },
    };

    Ok(CompiledExpr {
        expr: parse_quote! {
            ::tinc::__private::cel::CelValue::#op(
                #left,
                #right,
            )?
        },
        ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
    })
}

fn resolve_ternary(
    ctx: &Compiler,
    cond: &Expression,
    left: &Expression,
    right: &Expression,
) -> Result<CompiledExpr, CompileError> {
    let cond = ctx.resolve(cond)?;
    let left = ctx.resolve(left)?;
    if !left.ty.can_be_cel() {
        return Err(CompileError::TypeConversion {
            ty: Box::new(left.ty.clone()),
            message: "ternary operations must return CEL values".to_string(),
        });
    }

    let right = ctx.resolve(right)?;
    if !right.ty.can_be_cel() {
        return Err(CompileError::TypeConversion {
            ty: Box::new(right.ty.clone()),
            message: "ternary operations must return CEL values".to_string(),
        });
    }

    let cond = helpers::to_bool(cond);
    Ok(CompiledExpr {
        expr: parse_quote! {
            if (#cond) {
                ::tinc::__private::cel::CelValueConv::conv(#left)
            } else {
                ::tinc::__private::cel::CelValueConv::conv(#right)
            }
        },
        ty: CelType::CelValue,
    })
}

fn resolve_unary(ctx: &Compiler, op: &cel_parser::UnaryOp, expr: &Expression) -> Result<CompiledExpr, CompileError> {
    let expr = ctx.resolve(expr)?;
    match op {
        cel_parser::UnaryOp::Not => {
            let expr = helpers::to_bool(expr);
            Ok(CompiledExpr {
                expr: parse_quote! {
                    !(#expr)
                },
                ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
            })
        }
        cel_parser::UnaryOp::DoubleNot => Ok(helpers::to_bool(expr)),
        cel_parser::UnaryOp::Minus => {
            if !expr.ty.can_be_cel() {
                return Err(CompileError::TypeConversion {
                    ty: Box::new(expr.ty.clone()),
                    message: "cannot perform minus operation on a non-CEL value".to_string(),
                });
            }

            Ok(CompiledExpr {
                expr: parse_quote! {
                    ::tinc::__private::cel::CelValueConv::conv(#expr).neg()?
                },
                ty: expr.ty.clone(),
            })
        }
        cel_parser::UnaryOp::DoubleMinus => Ok(expr),
    }
}
