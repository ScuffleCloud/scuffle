use syn::parse_quote;

use super::CompiledExpr;
use crate::cel::codegen::{CelType, ProtoModifiedValueType, ProtoType, ProtoValueType};

pub fn to_bool(CompiledExpr { expr, ty }: CompiledExpr) -> CompiledExpr {
    match ty {
        CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::OneOf(_))) => CompiledExpr {
            expr: parse_quote! { true },
            ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
        },
        CelType::Proto(ProtoType::Modified(ProtoModifiedValueType::Optional(ty))) => {
            let value_to_bool = to_bool(CompiledExpr {
                expr: parse_quote! { ___to_bool_value },
                ty: CelType::Proto(ProtoType::Value(ty.clone())),
            });

            CompiledExpr {
                expr: parse_quote! {
                    match #expr {
                        Some(___to_bool_value) => #value_to_bool,
                        None => false,
                    }
                },
                ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
            }
        }
        CelType::Proto(ProtoType::Value(ProtoValueType::Message(_))) => CompiledExpr {
            expr: parse_quote! { true },
            ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
        },
        _ => CompiledExpr {
            expr: parse_quote! {
                ::tinc::__private::cel::to_bool(#expr)
            },
            ty: CelType::Proto(ProtoType::Value(ProtoValueType::Bool)),
        },
    }
}
