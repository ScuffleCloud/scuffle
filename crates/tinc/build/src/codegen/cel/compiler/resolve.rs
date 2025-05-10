use cel_parser::{ArithmeticOp, Atom, Expression, Member, RelationOp};
use quote::quote;
use syn::parse_quote;
use tinc_cel::CelValue;

use super::{CompileError, CompiledExpr, Compiler, CompilerCtx, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType};

pub(crate) fn resolve(ctx: &Compiler, expr: &Expression) -> Result<CompiledExpr, CompileError> {
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
    let left = ctx.resolve(left)?.into_bool(ctx);
    let right = ctx.resolve(right)?.into_bool(ctx);
    match (left, right) {
        (
            CompiledExpr::Constant(ConstantCompiledExpr { value: left }),
            CompiledExpr::Constant(ConstantCompiledExpr { value: right }),
        ) => Ok(CompiledExpr::constant(left.to_bool() && right.to_bool())),
        (CompiledExpr::Constant(ConstantCompiledExpr { value: const_value }), other)
        | (other, CompiledExpr::Constant(ConstantCompiledExpr { value: const_value })) => {
            if const_value.to_bool() {
                Ok(other)
            } else {
                Ok(CompiledExpr::constant(false))
            }
        }
        (left, right) => Ok(CompiledExpr::runtime(
            CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
            parse_quote! {
                (#left) && (#right)
            },
        )),
    }
}

fn resolve_arithmetic(
    ctx: &Compiler,
    left: &Expression,
    op: &ArithmeticOp,
    right: &Expression,
) -> Result<CompiledExpr, CompileError> {
    let left = ctx.resolve(left)?.into_cel()?;
    let right = ctx.resolve(right)?.into_cel()?;
    match (left, right) {
        (
            CompiledExpr::Constant(ConstantCompiledExpr { value: left }),
            CompiledExpr::Constant(ConstantCompiledExpr { value: right }),
        ) => match op {
            ArithmeticOp::Add => Ok(CompiledExpr::constant(CelValue::cel_add(left, right)?)),
            ArithmeticOp::Subtract => Ok(CompiledExpr::constant(CelValue::cel_sub(left, right)?)),
            ArithmeticOp::Divide => Ok(CompiledExpr::constant(CelValue::cel_div(left, right)?)),
            ArithmeticOp::Multiply => Ok(CompiledExpr::constant(CelValue::cel_mul(left, right)?)),
            ArithmeticOp::Modulus => Ok(CompiledExpr::constant(CelValue::cel_rem(left, right)?)),
        },
        (left, right) => {
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
    }
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
        .map(|item| ctx.resolve(item)?.into_cel())
        .collect::<Result<Vec<_>, _>>()?;

    if items.iter().any(|i| matches!(i, CompiledExpr::Runtime(_))) {
        Ok(CompiledExpr::runtime(
            CelType::CelValue,
            parse_quote! {
                ::tinc::__private::cel::CelValue::List(::std::iter::FromIterator::from_iter([
                    #(#items),*
                ]))
            },
        ))
    } else {
        Ok(CompiledExpr::constant(CelValue::List(
            items
                .into_iter()
                .map(|item| match item {
                    CompiledExpr::Constant(ConstantCompiledExpr { value }) => value,
                    _ => unreachable!(),
                })
                .collect(),
        )))
    }
}

fn resolve_map(ctx: &Compiler, items: &[(Expression, Expression)]) -> Result<CompiledExpr, CompileError> {
    let items = items
        .iter()
        .map(|(key, value)| {
            let key = ctx.resolve(key)?.into_cel()?;
            let value = ctx.resolve(value)?.into_cel()?;
            Ok((key, value))
        })
        .collect::<Result<Vec<_>, CompileError>>()?;

    if items
        .iter()
        .any(|(key, value)| matches!(key, CompiledExpr::Runtime(_)) || matches!(value, CompiledExpr::Runtime(_)))
    {
        let items = items.into_iter().map(|(key, value)| quote!((#key, #value)));
        Ok(CompiledExpr::runtime(
            CelType::CelValue,
            parse_quote! {
                ::tinc::__private::cel::CelValue::Map(::std::iter::FromIterator::from_iter([
                    #(#items),*
                ]))
            },
        ))
    } else {
        Ok(CompiledExpr::constant(CelValue::Map(
            items
                .into_iter()
                .map(|(key, value)| match (key, value) {
                    (
                        CompiledExpr::Constant(ConstantCompiledExpr { value: key }),
                        CompiledExpr::Constant(ConstantCompiledExpr { value }),
                    ) => (key, value),
                    _ => unreachable!(),
                })
                .collect(),
        )))
    }
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
                        ::tinc::__private::cel::CelValue::access(
                            #expr,
                            #attr
                        )?
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
                        ::tinc::__private::cel::map_access(
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
            let idx = ctx.resolve(idx)?.into_cel()?;
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
                        ::tinc::__private::cel::map_access(
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
    let left = ctx.resolve(left)?.into_bool(ctx);
    let right = ctx.resolve(right)?.into_bool(ctx);
    match (left, right) {
        (
            CompiledExpr::Constant(ConstantCompiledExpr { value: left }),
            CompiledExpr::Constant(ConstantCompiledExpr { value: right }),
        ) => Ok(CompiledExpr::constant(left.to_bool() || right.to_bool())),
        (CompiledExpr::Constant(ConstantCompiledExpr { value: const_value }), other)
        | (other, CompiledExpr::Constant(ConstantCompiledExpr { value: const_value })) => {
            if const_value.to_bool() {
                Ok(CompiledExpr::constant(true))
            } else {
                Ok(other)
            }
        }
        (left, right) => Ok(CompiledExpr::runtime(
            CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
            parse_quote! {
                (#left) || (#right)
            },
        )),
    }
}

fn resolve_relation(
    ctx: &Compiler,
    left: &Expression,
    op: &RelationOp,
    right: &Expression,
) -> Result<CompiledExpr, CompileError> {
    let left = ctx.resolve(left)?.into_cel()?;
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

    let right = right.into_cel()?;

    match (left, right) {
        (
            CompiledExpr::Constant(ConstantCompiledExpr { value: left }),
            CompiledExpr::Constant(ConstantCompiledExpr { value: right }),
        ) => match op {
            RelationOp::LessThan => Ok(CompiledExpr::constant(CelValue::cel_lt(left, right)?)),
            RelationOp::LessThanEq => Ok(CompiledExpr::constant(CelValue::cel_lte(left, right)?)),
            RelationOp::GreaterThan => Ok(CompiledExpr::constant(CelValue::cel_gt(left, right)?)),
            RelationOp::GreaterThanEq => Ok(CompiledExpr::constant(CelValue::cel_gte(left, right)?)),
            RelationOp::Equals => Ok(CompiledExpr::constant(CelValue::cel_eq(left, right)?)),
            RelationOp::NotEquals => Ok(CompiledExpr::constant(CelValue::cel_neq(left, right)?)),
            RelationOp::In => Ok(CompiledExpr::constant(CelValue::cel_in(left, right)?)),
        },
        (left, right) => {
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
    }
}

fn resolve_ternary(
    ctx: &Compiler,
    cond: &Expression,
    left: &Expression,
    right: &Expression,
) -> Result<CompiledExpr, CompileError> {
    let cond = ctx.resolve(cond)?.into_bool(ctx);
    let left = ctx.resolve(left)?.into_cel()?;
    let right = ctx.resolve(right)?.into_cel()?;

    match cond {
        CompiledExpr::Constant(ConstantCompiledExpr { value: cond }) => {
            if cond.to_bool() {
                Ok(left)
            } else {
                Ok(right)
            }
        }
        cond => Ok(CompiledExpr::runtime(
            CelType::CelValue,
            parse_quote! {
                if (#cond) {
                    #left
                } else {
                    #right
                }
            },
        )),
    }
}

fn resolve_unary(ctx: &Compiler, op: &cel_parser::UnaryOp, expr: &Expression) -> Result<CompiledExpr, CompileError> {
    let expr = ctx.resolve(expr)?;
    match op {
        cel_parser::UnaryOp::Not => {
            let expr = expr.into_bool(ctx);
            match expr {
                CompiledExpr::Constant(ConstantCompiledExpr { value: expr }) => Ok(CompiledExpr::constant(!expr.to_bool())),
                expr => Ok(CompiledExpr::runtime(
                    CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                    parse_quote! {
                        !(::tinc::__private::cel::to_bool(#expr))
                    },
                )),
            }
        }
        cel_parser::UnaryOp::DoubleNot => Ok(expr.into_bool(ctx)),
        cel_parser::UnaryOp::Minus => {
            let expr = expr.into_cel()?;
            match expr {
                CompiledExpr::Constant(ConstantCompiledExpr { value: expr }) => {
                    Ok(CompiledExpr::constant(CelValue::cel_neg(expr)?))
                }
                expr => Ok(CompiledExpr::runtime(
                    CelType::CelValue,
                    parse_quote! {
                        ::tinc::__private::cel::CelValue::cel_neg(#expr)?
                    },
                )),
            }
        }
        cel_parser::UnaryOp::DoubleMinus => Ok(expr),
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use cel_parser::parse as parse_cel;

    use super::*;
    use crate::types::ProtoTypeRegistry;

    #[test]
    fn test_resolve_atom_int() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);
        let expr = parse_cel("1").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            1,
                        ),
                    ),
                },
            ),
        )
        ");
    }

    #[test]
    fn test_resolve_atom_uint() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);
        let expr = parse_cel("3u").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        U64(
                            3,
                        ),
                    ),
                },
            ),
        )
        ");
    }

    #[test]
    fn test_resolve_atom_float() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);
        let expr = parse_cel("1.23").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        F64(
                            1.23,
                        ),
                    ),
                },
            ),
        )
        ");
    }

    #[test]
    fn test_resolve_atom_string_bytes_bool_null() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let expr_str = parse_cel("\"foo\"").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_str), @r#"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: String(
                        Owned(
                            "foo",
                        ),
                    ),
                },
            ),
        )
        "#);

        let expr_bytes = parse_cel("b\"hi\"").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_bytes), @r#"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Bytes(
                        Owned(
                            b"hi",
                        ),
                    ),
                },
            ),
        )
        "#);

        let expr_bool = parse_cel("true").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_bool), @r"
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

        let expr_null = parse_cel("null").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_null), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Null,
                },
            ),
        )
        ");
    }

    #[test]
    fn test_resolve_arithmetic_constant() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let expr = parse_cel("10 + 5").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            15,
                        ),
                    ),
                },
            ),
        )
        ");

        let expr = parse_cel("10 - 4").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            6,
                        ),
                    ),
                },
            ),
        )
        ");

        let expr = parse_cel("6 * 7").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            42,
                        ),
                    ),
                },
            ),
        )
        ");

        let expr = parse_cel("20 / 4").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            5,
                        ),
                    ),
                },
            ),
        )
        ");

        let expr = parse_cel("10 % 3").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            1,
                        ),
                    ),
                },
            ),
        )
        ");
    }

    #[test]
    fn test_resolve_relation_constant() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let expr = parse_cel("1 < 2").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
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
        let expr = parse_cel("1 <= 1").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
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
        let expr = parse_cel("2 > 1").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
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
        let expr = parse_cel("2 >= 2").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
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
        let expr = parse_cel("1 == 1").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
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
        let expr = parse_cel("1 != 2").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
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
        let expr = parse_cel("1 in [1, 2, 3]").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr), @r"
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
    fn test_resolve_boolean_constant() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let expr_and = parse_cel("true && false").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_and), @r"
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

        let expr_or = parse_cel("true || false").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_or), @r"
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
    fn test_resolve_unary_constant() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let expr_not = parse_cel("!false").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_not), @r"
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

        let expr_double_not = parse_cel("!!true").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_double_not), @r"
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

        let expr_neg = parse_cel("-5").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_neg), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            -5,
                        ),
                    ),
                },
            ),
        )
        ");

        let expr_double_neg = parse_cel("--5").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_double_neg), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            5,
                        ),
                    ),
                },
            ),
        )
        ");
    }

    #[test]
    fn test_resolve_ternary_constant() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let expr_true = parse_cel("true ? 1 : 2").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_true), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            1,
                        ),
                    ),
                },
            ),
        )
        ");

        let expr_false = parse_cel("false ? 1 : 2").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_false), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            2,
                        ),
                    ),
                },
            ),
        )
        ");
    }

    #[test]
    fn test_resolve_list_map_constant() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let expr_list = parse_cel("[1, 2, 3]").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_list), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: List(
                        [
                            Number(
                                I64(
                                    1,
                                ),
                            ),
                            Number(
                                I64(
                                    2,
                                ),
                            ),
                            Number(
                                I64(
                                    3,
                                ),
                            ),
                        ],
                    ),
                },
            ),
        )
        ");

        let expr_map = parse_cel("{'a': 1, 'b': 2}").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_map), @r#"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Map(
                        [
                            (
                                String(
                                    Owned(
                                        "a",
                                    ),
                                ),
                                Number(
                                    I64(
                                        1,
                                    ),
                                ),
                            ),
                            (
                                String(
                                    Owned(
                                        "b",
                                    ),
                                ),
                                Number(
                                    I64(
                                        2,
                                    ),
                                ),
                            ),
                        ],
                    ),
                },
            ),
        )
        "#);
    }

    #[test]
    fn test_resolve_negative_variable() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let mut compiler = Compiler::new(&registry);

        compiler.add_variable("x", CompiledExpr::constant(CelValue::Number(1.into())));

        let expr_list = parse_cel("-x").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_list), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            -1,
                        ),
                    ),
                },
            ),
        )
        ");
    }

    #[test]
    fn test_resolve_access() {
        let registry = ProtoTypeRegistry::new(crate::Mode::Prost, crate::extern_paths::ExternPaths::new(crate::Mode::Prost));
        let compiler = Compiler::new(&registry);

        let expr_list = parse_cel("[1, 2, 3][2]").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_list), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            3,
                        ),
                    ),
                },
            ),
        )
        ");

        let expr_map = parse_cel("({'a': 1, 'b': 2}).a").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_map), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            1,
                        ),
                    ),
                },
            ),
        )
        ");

        let expr_map = parse_cel("({'a': 1, 'b': 2})['b']").unwrap();
        insta::assert_debug_snapshot!(resolve(&compiler, &expr_map), @r"
        Ok(
            Constant(
                ConstantCompiledExpr {
                    value: Number(
                        I64(
                            2,
                        ),
                    ),
                },
            ),
        )
        ");
    }
}
