use quote::quote;
use syn::parse_quote;
use tinc_cel::CelValue;

use super::Function;
use crate::codegen::cel::compiler::{
    CompileError, CompiledExpr, Compiler, CompilerCtx, CompilerTarget, ConstantCompiledExpr, RuntimeCompiledExpr,
};
use crate::codegen::cel::types::CelType;

#[derive(Debug, Clone, Default)]
pub struct String;

fn cel_to_string(ctx: &Compiler, value: &CelValue<'static>) -> CompiledExpr {
    match value {
        CelValue::List(list) => {
            let items: Vec<_> = list.iter().map(|item| cel_to_string(ctx, item)).collect();
            if items.iter().any(|item| matches!(item, CompiledExpr::Runtime(_))) {
                CompiledExpr::runtime(
                    CelType::CelValue,
                    parse_quote!({
                        ::tinc::__private::cel::CelValue::cel_to_string(::tinc::__private::cel::CelValue::List([
                            #(#items),*
                        ].into_iter().collect()))
                    }),
                )
            } else {
                CompiledExpr::constant(CelValue::cel_to_string(CelValue::List(
                    items
                        .into_iter()
                        .map(|i| match i {
                            CompiledExpr::Constant(ConstantCompiledExpr { value }) => value,
                            _ => unreachable!(),
                        })
                        .collect(),
                )))
            }
        }
        CelValue::Map(map) => {
            let items: Vec<_> = map
                .iter()
                .map(|(key, value)| (cel_to_string(ctx, key), cel_to_string(ctx, value)))
                .collect();
            if items
                .iter()
                .any(|(key, value)| matches!(key, CompiledExpr::Runtime(_)) || matches!(value, CompiledExpr::Runtime(_)))
            {
                let items = items.iter().map(|(key, value)| quote!((#key, #value)));
                CompiledExpr::runtime(
                    CelType::CelValue,
                    parse_quote!({
                        ::tinc::__private::cel::CelValue::cel_to_string(::tinc::__private::cel::CelValue::Map([
                            #(#items),*
                        ].into_iter().collect()))
                    }),
                )
            } else {
                CompiledExpr::constant(CelValue::cel_to_string(CelValue::Map(
                    items
                        .into_iter()
                        .map(|i| match i {
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
        CelValue::Enum(cel_enum) => {
            let Some((proto_name, proto_enum)) = ctx
                .registry()
                .get_enum(&cel_enum.tag)
                .and_then(|e| e.variants.iter().find(|(_, v)| v.value == cel_enum.value))
            else {
                return CompiledExpr::constant(CelValue::cel_to_string(cel_enum.value));
            };

            let json_name = &proto_enum.options.json_name;

            match ctx.target() {
                Some(CompilerTarget::Json) => CompiledExpr::constant(CelValue::String(json_name.clone().into())),
                Some(CompilerTarget::Proto) => CompiledExpr::constant(CelValue::String(proto_name.clone().into())),
                None => CompiledExpr::runtime(
                    CelType::CelValue,
                    parse_quote! {
                        match ::tinc::__private::cel::CelMode::current() {
                            ::tinc::__private::cel::CelMode::Json => ::tinc::__private::cel::CelValueConv::conv(#json_name),
                            ::tinc::__private::cel::CelMode::Proto => ::tinc::__private::cel::CelValueConv::conv(#proto_name),
                        }
                    },
                ),
            }
        }
        v @ (CelValue::Bool(_)
        | CelValue::Bytes(_)
        | CelValue::Duration(_)
        | CelValue::Null
        | CelValue::Number(_)
        | CelValue::String(_)
        | CelValue::Timestamp(_)) => CompiledExpr::constant(CelValue::cel_to_string(v.clone())),
    }
}

impl Function for String {
    fn name(&self) -> &'static str {
        "string"
    }

    fn syntax(&self) -> &'static str {
        "<this>.string()"
    }

    fn compile(&self, mut ctx: CompilerCtx) -> Result<CompiledExpr, CompileError> {
        let Some(this) = ctx.this.take() else {
            return Err(CompileError::syntax("missing this", self));
        };

        if !ctx.args.is_empty() {
            return Err(CompileError::syntax("takes no arguments", self));
        }

        match this.to_cel()? {
            CompiledExpr::Constant(ConstantCompiledExpr { value }) => Ok(cel_to_string(&ctx, &value)),
            CompiledExpr::Runtime(RuntimeCompiledExpr { expr, .. }) => Ok(CompiledExpr::runtime(
                CelType::CelValue,
                parse_quote!(::tinc::__private::cel::CelValue::cel_to_string(#expr)),
            )),
        }
    }
}
