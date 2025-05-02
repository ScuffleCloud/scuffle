use cel_parser::{ArithmeticOp, Atom, Expression, Member, RelationOp};
use quote::quote;
use syn::parse_quote;

use super::{CompileError, CompiledExpr, Compiler, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
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
    let left = ctx.resolve(left)?.to_bool(ctx);
    let right = ctx.resolve(right)?.to_bool(ctx);
    Ok(CompiledExpr::runtime(
        CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
        parse_quote! {
            (#left) && (#right)
        },
    ))
}

fn resolve_arithmetic(
    ctx: &Compiler,
    left: &Expression,
    op: &ArithmeticOp,
    right: &Expression,
) -> Result<CompiledExpr, CompileError> {
    let left = ctx.resolve(left)?.to_cel()?;
    let right = ctx.resolve(right)?.to_cel()?;

    let op = match op {
        ArithmeticOp::Add => quote! { cel_add },
        ArithmeticOp::Subtract => quote! { cel_sub },
        ArithmeticOp::Divide => quote! { cel_div },
        ArithmeticOp::Multiply => quote! { cel_mul },
        ArithmeticOp::Modulus => quote! { cel_rem },
    };

    Ok(CompiledExpr::runtime(
        CelType::CelValue,
        parse_quote! {
            ::tinc::__private::cel::CelValue::#op(
                #right,
                #left,
            )?
        },
    ))
}

fn resolve_atom(_: &Compiler, atom: &Atom) -> Result<CompiledExpr, CompileError> {
    match atom {
        Atom::Int(v) => Ok(CompiledExpr::constant(v)),
        Atom::UInt(v) => Ok(CompiledExpr::constant(v)),
        Atom::Float(v) => Ok(CompiledExpr::constant(v)),
        Atom::String(v) => Ok(CompiledExpr::constant(tinc_cel::CelValue::String(v.to_string().into()))),
        Atom::Bytes(v) => Ok(CompiledExpr::constant(tinc_cel::CelValue::Bytes(v.to_vec().into()))),
        Atom::Bool(v) => Ok(CompiledExpr::constant(v)),
        Atom::Null => Ok(CompiledExpr::constant(tinc_cel::CelValue::Null)),
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

    func.compile(CompilerCtx::new(ctx.child(), this, args))
}

fn resolve_ident(ctx: &Compiler, ident: &str) -> Result<CompiledExpr, CompileError> {
    ctx.get_variable(ident)
        .cloned()
        .ok_or_else(|| CompileError::VariableNotFound(ident.to_owned()))
}

fn resolve_list(ctx: &Compiler, items: &[Expression]) -> Result<CompiledExpr, CompileError> {
    let items = items
        .iter()
        .map(|item| ctx.resolve(item)?.to_cel())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CompiledExpr::runtime(
        CelType::CelValue,
        parse_quote! {
            ::tinc::__private::cel::CelValue::List(::std::iter::FromIterator::from_iter([
                #(#items),*
            ]))
        },
    ))
}

fn resolve_map(ctx: &Compiler, items: &[(Expression, Expression)]) -> Result<CompiledExpr, CompileError> {
    dbg!(items);

    let items = items
        .iter()
        .map(|(key, value)| {
            let key = ctx.resolve(key)?.to_cel()?;
            let value = ctx.resolve(value)?.to_cel()?;
            Ok(quote! {
                (
                    #key,
                    #value,
                )
            })
        })
        .collect::<Result<Vec<_>, CompileError>>()?;

    Ok(CompiledExpr::runtime(
        CelType::CelValue,
        parse_quote! {
            ::tinc::__private::cel::CelValueConv::Map(::std::iter::FromIterator::from_iter([
                #(#items),*
            ]))
        },
    ))
}

fn resolve_member(ctx: &Compiler, expr: &Expression, member: &Member) -> Result<CompiledExpr, CompileError> {
    let expr = ctx.resolve(expr)?;
    match member {
        Member::Attribute(attr) => {
            let attr = attr.as_str();
            match &expr {
                CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr,
                    ty: CelType::CelValue,
                }) => Ok(CompiledExpr::runtime(
                    CelType::CelValue,
                    parse_quote! {
                        (#expr).access(#attr)?
                    },
                )),
                CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr,
                    ty:
                        ty @ CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Message(
                            full_name,
                        )))),
                }) => {
                    let msg = ctx
                        .registry()
                        .get_message(full_name)
                        .ok_or_else(|| CompileError::MissingMessage(full_name.clone()))?;

                    let field_ty = msg.fields.get(attr).ok_or_else(|| CompileError::MemberAccess {
                        ty: Box::new(ty.clone()),
                        message: format!("message {} does not have field {}", msg.full_name, attr),
                    })?;

                    let field_ident = field_ty.rust_ident();

                    Ok(CompiledExpr::runtime(
                        CelType::Proto(field_ty.ty.clone()),
                        parse_quote! {
                            match (#expr) {
                                Some(value) => &value.#field_ident,
                                None => return Err(::tinc::__private::cel::CelError::BadAccess {
                                    member: ::tinc::__private::cel::CelValue::String(::tinc::__private::cel::CelString::Borrowed(#attr)),
                                    container: ::tinc::__private::cel::CelValue::Null,
                                }),
                            }
                        },
                    ))
                }
                CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr,
                    ty: ty @ CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::OneOf(oneof))),
                }) => {
                    let field_ty = oneof.fields.get(attr).ok_or_else(|| CompileError::MemberAccess {
                        ty: Box::new(ty.clone()),
                        message: format!("oneof {} does not have field {}", oneof.full_name, attr),
                    })?;

                    let field_ident = field_ty.rust_ident();

                    Ok(CompiledExpr::runtime(
                        CelType::Proto(ProtoType::Value(field_ty.ty.clone())),
                        parse_quote! {
                            match (#expr) {
                                Some(value) => &value.#field_ident,
                                None => return Err(::tinc::__private::cel::CelError::BadAccess {
                                    member: ::tinc::__private::cel::CelValue::String(::tinc::__private::cel::CelString::Borrowed(#attr)),
                                    container: ::tinc::__private::cel::CelValue::Null,
                                }),
                            }
                        },
                    ))
                }
                CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr,
                    ty: ty @ CelType::Proto(ProtoType::Value(ProtoValueType::Message(full_name))),
                }) => {
                    let msg = ctx
                        .registry()
                        .get_message(full_name)
                        .ok_or_else(|| CompileError::MissingMessage(full_name.clone()))?;
                    let field_ty = msg.fields.get(attr).ok_or_else(|| CompileError::MemberAccess {
                        ty: Box::new(ty.clone()),
                        message: format!("message {} does not have field {}", msg.full_name, attr),
                    })?;

                    let field_ident = field_ty.rust_ident();

                    Ok(CompiledExpr::runtime(
                        CelType::Proto(field_ty.ty.clone()),
                        parse_quote! {
                            &(#expr).#field_ident,
                        },
                    ))
                }
                CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr,
                    ty: CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(ProtoValueType::String, value_ty))),
                }) => Ok(CompiledExpr::runtime(
                    CelType::Proto(ProtoType::Value(value_ty.clone())),
                    parse_quote! {
                        ::tinc::__private::cel::CelValueConv::map_access(
                            #expr,
                            #attr,
                        )?
                    },
                )),
                CompiledExpr::Runtime(RuntimeCompiledExpr { ty, .. }) => Err(CompileError::MemberAccess {
                    ty: Box::new(ty.clone()),
                    message: "can only access attributes on messages and maps with string keys".to_string(),
                }),
                CompiledExpr::Constant(ConstantCompiledExpr { value: container }) => {
                    Ok(CompiledExpr::constant(tinc_cel::CelValue::cel_access(container, attr)?))
                }
            }
        }
        Member::Index(idx) => {
            let idx = ctx.resolve(idx)?.to_cel()?;
            match (expr, idx) {
                (
                    expr @ CompiledExpr::Runtime(RuntimeCompiledExpr {
                        ty: CelType::CelValue, ..
                    }),
                    idx,
                )
                | (expr @ CompiledExpr::Constant(_), idx @ CompiledExpr::Runtime(_)) => Ok(CompiledExpr::runtime(
                    CelType::CelValue,
                    parse_quote! {
                        ::tinc::__private::cel::CelValue::cel_access(#expr, #idx)?
                    },
                )),
                (
                    CompiledExpr::Runtime(RuntimeCompiledExpr {
                        expr,
                        ty: CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(item_ty))),
                    }),
                    idx,
                ) => Ok(CompiledExpr::runtime(
                    CelType::Proto(ProtoType::Value(item_ty.clone())),
                    parse_quote! {
                        ::tinc::__private::cel::CelValueConv::array_access(
                            #expr,
                            #idx,
                        )?
                    },
                )),
                (
                    CompiledExpr::Runtime(RuntimeCompiledExpr {
                        expr,
                        ty: CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(_, value_ty))),
                    }),
                    idx,
                ) => Ok(CompiledExpr::runtime(
                    CelType::Proto(ProtoType::Value(value_ty.clone())),
                    parse_quote! {
                        ::tinc::__private::cel::CelValueConv::map_access(
                            #expr,
                            #idx,
                        )?
                    },
                )),
                (CompiledExpr::Runtime(RuntimeCompiledExpr { ty, .. }), _) => Err(CompileError::MemberAccess {
                    ty: Box::new(ty.clone()),
                    message: "cannot index into non-repeated and non-map values".to_string(),
                }),
                (
                    CompiledExpr::Constant(ConstantCompiledExpr { value: container }),
                    CompiledExpr::Constant(ConstantCompiledExpr { value: idx }),
                ) => Ok(CompiledExpr::constant(tinc_cel::CelValue::cel_access(container, idx)?)),
            }
        }
        Member::Fields(_) => Err(CompileError::NotImplemented),
    }
}

fn resolve_or(ctx: &Compiler, left: &Expression, right: &Expression) -> Result<CompiledExpr, CompileError> {
    let left = ctx.resolve(left)?.to_bool(ctx);
    let right = ctx.resolve(right)?.to_bool(ctx);
    Ok(CompiledExpr::runtime(
        CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
        parse_quote! {
            (#left) || (#right)
        },
    ))
}

fn resolve_relation(
    ctx: &Compiler,
    left: &Expression,
    op: &RelationOp,
    right: &Expression,
) -> Result<CompiledExpr, CompileError> {
    let left = ctx.resolve(left)?.to_cel()?;
    let right = ctx.resolve(right)?;
    if let (
        RelationOp::In,
        CompiledExpr::Runtime(RuntimeCompiledExpr {
            ty:
                right_ty @ CelType::Proto(ProtoType::Modified(
                    ProtoModifiedValueType::Repeated(item) | ProtoModifiedValueType::Map(item, _),
                )),
            ..
        }),
    ) = (op, &right)
    {
        if !matches!(item, ProtoValueType::Message { .. }) {
            let op = match &right_ty {
                CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(_))) => {
                    quote! { array_contains }
                }
                CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(_, _))) => quote! { map_contains },
                _ => unreachable!(),
            };

            return Ok(CompiledExpr::runtime(
                CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                parse_quote! {
                    ::tinc::__private::cel::#op(
                        #right,
                        #left,
                    )
                },
            ));
        }
    }

    let right = right.to_cel()?;

    let op = match op {
        RelationOp::LessThan => quote! { cel_lt },
        RelationOp::LessThanEq => quote! { cel_lte },
        RelationOp::GreaterThan => quote! { cel_gt },
        RelationOp::GreaterThanEq => quote! { cel_gte },
        RelationOp::Equals => quote! { cel_eq },
        RelationOp::NotEquals => quote! { cel_neq },
        RelationOp::In => quote! { cel_in },
    };

    Ok(CompiledExpr::runtime(
        CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
        parse_quote! {
            ::tinc::__private::cel::CelValue::#op(
                #left,
                #right,
            )?
        },
    ))
}

fn resolve_ternary(
    ctx: &Compiler,
    cond: &Expression,
    left: &Expression,
    right: &Expression,
) -> Result<CompiledExpr, CompileError> {
    let cond = ctx.resolve(cond)?.to_bool(ctx);
    let left = ctx.resolve(left)?.to_cel()?;
    let right = ctx.resolve(right)?.to_cel()?;

    Ok(CompiledExpr::runtime(
        CelType::CelValue,
        parse_quote! {
            if (#cond) {
                #left
            } else {
                #right
            }
        },
    ))
}

fn resolve_unary(ctx: &Compiler, op: &cel_parser::UnaryOp, expr: &Expression) -> Result<CompiledExpr, CompileError> {
    let expr = ctx.resolve(expr)?;
    match op {
        cel_parser::UnaryOp::Not => {
            let expr = expr.to_bool(ctx);
            Ok(CompiledExpr::runtime(
                CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                parse_quote! {
                    !(::tinc::__private::cel::to_bool(#expr))
                },
            ))
        }
        cel_parser::UnaryOp::DoubleNot => Ok(expr.to_bool(ctx)),
        cel_parser::UnaryOp::Minus => {
            let expr = expr.to_cel()?;

            Ok(CompiledExpr::runtime(
                CelType::CelValue,
                parse_quote! {
                    ::tinc::__private::cel::CelValue::cel_neg(#expr)
                },
            ))
        }
        cel_parser::UnaryOp::DoubleMinus => Ok(expr),
    }
}
