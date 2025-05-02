use syn::parse_quote;

use super::{CompileError, CompiledExpr};
use crate::codegen::cel::types::CelType;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType, ProtoWellKnownType};

impl CompiledExpr {
    pub fn to_bool(self) -> CompiledExpr {
        match &self.ty {
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::OneOf(_))) => CompiledExpr {
                expr: parse_quote! { (#self).is_some() },
                ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
            },
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Optional(ty))) => {
                let value_to_bool = CompiledExpr {
                    expr: parse_quote! { ___to_bool_value },
                    ty: CelType::Proto(ProtoType::Value(ty.clone())),
                }
                .to_bool();

                CompiledExpr {
                    expr: parse_quote! {
                        match #self {
                            Some(___to_bool_value) => #value_to_bool,
                            None => false,
                        }
                    },
                    ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
                }
            }
            CelType::Proto(ProtoType::Value(ProtoValueType::Message { .. })) => CompiledExpr {
                expr: parse_quote! { true },
                ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
            },
            _ => CompiledExpr {
                expr: parse_quote! {
                    ::tinc::__private::cel::to_bool(#self)
                },
                ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
            },
        }
    }

    pub fn to_cel(self) -> Result<CompiledExpr, CompileError> {
        let CompiledExpr { expr, ty } = self;
        match ty {
            CelType::CelValue => Ok(CompiledExpr { expr, ty }),
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
            )) => Ok(CompiledExpr {
                expr: parse_quote! {
                    ::tinc::__private::cel::CelValueConv::conv(#expr)
                },
                ty: CelType::CelValue,
            }),
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Map(key_ty, value_ty))) => {
                let key_to_cel = CompiledExpr {
                    expr: parse_quote!(key),
                    ty: CelType::Proto(ProtoType::Value(key_ty)),
                }
                .to_cel()?;

                let value_to_cel = CompiledExpr {
                    expr: parse_quote!(value),
                    ty: CelType::Proto(ProtoType::Value(value_ty)),
                }
                .to_cel()?;

                Ok(CompiledExpr {
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
                })
            }
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Optional(some_ty))) => {
                let some_to_cel = CompiledExpr {
                    expr: parse_quote!(item),
                    ty: CelType::Proto(ProtoType::Value(some_ty)),
                }
                .to_cel()?;

                Ok(CompiledExpr {
                    expr: parse_quote! {{
                        match (#expr) {
                            ::core::option::Option::Some(item) => #some_to_cel,
                            ::core::option::Option::None => ::tinc::__private::cel::CelValue::Null,
                        }
                    }},
                    ty: CelType::CelValue,
                })
            }
            CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Repeated(item_ty))) => {
                let item_to_cel = CompiledExpr {
                    expr: parse_quote!(item),
                    ty: CelType::Proto(ProtoType::Value(item_ty)),
                }
                .to_cel()?;

                Ok(CompiledExpr {
                    expr: parse_quote! {
                        ::tinc::__private::cel::CelValue::List((#expr).into_iter().map(|item| #item_to_cel).collect())
                    },
                    ty: CelType::CelValue,
                })
            }
            CelType::Proto(ProtoType::Value(ProtoValueType::Enum(path))) => {
                let path = path.as_ref();
                Ok(CompiledExpr {
                    expr: parse_quote! {
                        ::tinc::__private::cel::CelValue::Enum(::tinc::__private::cel::CelEnum {
                            tag: ::tinc::__private::cel::CelString::Borrowed(#path),
                            value: #expr,
                        })
                    },
                    ty: CelType::CelValue,
                })
            }
            // Not sure how to represent oneofs in cel.
            ty @ CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::OneOf(_))) => {
                Err(CompileError::TypeConversion {
                    ty: Box::new(ty),
                    message: "oneofs cannot be converted into cel types".into(),
                })
            }
            // Nor messages
            CelType::Proto(ProtoType::Value(ProtoValueType::Message(_))) => Err(CompileError::TypeConversion {
                ty: Box::new(ty),
                message: "message types cannot be converted into cel types".into(),
            }),
            // Currently any is not supported.
            CelType::Proto(ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Any))) => {
                Err(CompileError::TypeConversion {
                    ty: Box::new(ty),
                    message: "any cannot be converted into cel types".into(),
                })
            }
        }
    }
}
