use syn::parse_quote;
use tinc_cel::CelValue;

use super::{CompileError, CompiledExpr, Compiler, ConstantCompiledExpr, RuntimeCompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType, ProtoWellKnownType};

impl CompiledExpr {
    pub(crate) fn into_bool(self, compiler: &Compiler) -> CompiledExpr {
        match &self {
            CompiledExpr::Runtime(RuntimeCompiledExpr {
                expr,
                ty: CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::OneOf(_))),
            }) => CompiledExpr::Runtime(RuntimeCompiledExpr {
                expr: parse_quote! { (#expr).is_some() },
                ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
            }),
            CompiledExpr::Runtime(RuntimeCompiledExpr {
                expr,
                ty: CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Optional(ty))),
            }) => {
                let value_to_bool = CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr: parse_quote! { ___to_bool_value },
                    ty: CelType::Proto(ProtoType::Value(ty.clone())),
                })
                .into_bool(compiler);

                CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr: parse_quote! {
                        match #expr {
                            Some(___to_bool_value) => #value_to_bool,
                            None => false,
                        }
                    },
                    ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                })
            }
            CompiledExpr::Runtime(RuntimeCompiledExpr {
                ty: CelType::Proto(ProtoType::Value(ProtoValueType::Message { .. })),
                ..
            }) => CompiledExpr::Constant(ConstantCompiledExpr {
                value: tinc_cel::CelValue::Bool(true),
            }),
            CompiledExpr::Runtime(RuntimeCompiledExpr { expr, .. }) => CompiledExpr::Runtime(RuntimeCompiledExpr {
                expr: parse_quote! {
                    ::tinc::__private::cel::to_bool(#expr)
                },
                ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
            }),
            CompiledExpr::Constant(ConstantCompiledExpr {
                value: CelValue::Enum(cel_enum),
            }) => CompiledExpr::Constant(ConstantCompiledExpr {
                value: tinc_cel::CelValue::Bool(
                    compiler
                        .registry
                        .get_enum(&cel_enum.tag)
                        .is_some_and(|e| e.variants.values().any(|v| v.value == cel_enum.value)),
                ),
            }),
            CompiledExpr::Constant(ConstantCompiledExpr { value }) => CompiledExpr::Constant(ConstantCompiledExpr {
                value: tinc_cel::CelValue::Bool(value.to_bool()),
            }),
        }
    }

    pub(crate) fn into_cel(self) -> Result<CompiledExpr, CompileError> {
        match self {
            CompiledExpr::Runtime(RuntimeCompiledExpr {
                expr,
                ty: ty @ CelType::CelValue,
            }) => Ok(CompiledExpr::Runtime(RuntimeCompiledExpr { expr, ty })),
            CompiledExpr::Runtime(RuntimeCompiledExpr {
                expr,
                ty:
                    CelType::Proto(ProtoType::Value(
                        ProtoValueType::Bool
                        | ProtoValueType::Bytes
                        | ProtoValueType::Double
                        | ProtoValueType::Float
                        | ProtoValueType::Int32
                        | ProtoValueType::Int64
                        | ProtoValueType::String
                        | ProtoValueType::UInt32
                        | ProtoValueType::UInt64
                        | ProtoValueType::WellKnown(
                            ProtoWellKnownType::Duration
                            | ProtoWellKnownType::Empty
                            | ProtoWellKnownType::List
                            | ProtoWellKnownType::Struct
                            | ProtoWellKnownType::Timestamp
                            | ProtoWellKnownType::Value,
                        ),
                    )),
            }) => Ok(CompiledExpr::Runtime(RuntimeCompiledExpr {
                expr: parse_quote! {
                    ::tinc::__private::cel::CelValueConv::conv(#expr)
                },
                ty: CelType::CelValue,
            })),
            CompiledExpr::Runtime(RuntimeCompiledExpr {
                expr,
                ty: CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(key_ty, value_ty))),
            }) => {
                let key_to_cel = CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr: parse_quote!(key),
                    ty: CelType::Proto(ProtoType::Value(key_ty)),
                })
                .into_cel()?;

                let value_to_cel = CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr: parse_quote!(value),
                    ty: CelType::Proto(ProtoType::Value(value_ty)),
                })
                .into_cel()?;

                Ok(CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr: parse_quote! {
                        ::tinc::__private::cel::CelValue::Map(
                            (#expr).into_iter().map(|(key, value)| {
                                (
                                    #key_to_cel,
                                    #value_to_cel,
                                )
                            }).collect()
                        )
                    },
                    ty: CelType::CelValue,
                }))
            }
            CompiledExpr::Runtime(RuntimeCompiledExpr {
                expr,
                ty: CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Optional(some_ty))),
            }) => {
                let some_to_cel = CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr: parse_quote!(item),
                    ty: CelType::Proto(ProtoType::Value(some_ty)),
                })
                .into_cel()?;

                Ok(CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr: parse_quote! {{
                        match (#expr) {
                            ::core::option::Option::Some(item) => #some_to_cel,
                            ::core::option::Option::None => ::tinc::__private::cel::CelValue::Null,
                        }
                    }},
                    ty: CelType::CelValue,
                }))
            }
            CompiledExpr::Runtime(RuntimeCompiledExpr {
                expr,
                ty: CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(item_ty))),
            }) => {
                let item_to_cel = CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr: parse_quote!(item),
                    ty: CelType::Proto(ProtoType::Value(item_ty)),
                })
                .into_cel()?;

                Ok(CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr: parse_quote! {
                        ::tinc::__private::cel::CelValue::List((#expr).into_iter().map(|item| #item_to_cel).collect())
                    },
                    ty: CelType::CelValue,
                }))
            }
            CompiledExpr::Runtime(RuntimeCompiledExpr {
                expr,
                ty: CelType::Proto(ProtoType::Value(ProtoValueType::Enum(path))),
            }) => {
                let path = path.as_ref();
                Ok(CompiledExpr::Runtime(RuntimeCompiledExpr {
                    expr: parse_quote! {
                        ::tinc::__private::cel::CelValue::cel_to_enum(
                            #expr,
                            #path,
                        )?
                    },
                    ty: CelType::CelValue,
                }))
            }
            // Not sure how to represent oneofs in cel.
            CompiledExpr::Runtime(RuntimeCompiledExpr {
                ty: ty @ CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::OneOf(_))),
                ..
            }) => Err(CompileError::TypeConversion {
                ty: Box::new(ty),
                message: "oneofs cannot be converted into cel types".into(),
            }),
            // Nor messages
            CompiledExpr::Runtime(RuntimeCompiledExpr {
                ty: ty @ CelType::Proto(ProtoType::Value(ProtoValueType::Message(_))),
                ..
            }) => Err(CompileError::TypeConversion {
                ty: Box::new(ty),
                message: "message types cannot be converted into cel types".into(),
            }),
            // Currently any is not supported.
            CompiledExpr::Runtime(RuntimeCompiledExpr {
                ty: ty @ CelType::Proto(ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Any))),
                ..
            }) => Err(CompileError::TypeConversion {
                ty: Box::new(ty),
                message: "any cannot be converted into cel types".into(),
            }),
            CompiledExpr::Constant(c) => Ok(CompiledExpr::Constant(c)),
        }
    }
}
