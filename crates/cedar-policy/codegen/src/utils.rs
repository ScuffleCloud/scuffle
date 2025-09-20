use cedar_policy_core::ast::Name;
use cedar_policy_core::validator::RawName;
use cedar_policy_core::validator::json_schema::{self, Fragment};
use heck::{ToSnakeCase, ToUpperCamelCase};

use crate::cedar_namespace::CedarNamespace;
use crate::codegen::Codegen;
use crate::error::CodegenError;
use crate::types::{CedarRef, CedarType, CedarTypeStructField, NamespaceId};
use crate::{CodegenResult, Config};

/// Creates a NamespaceId from an optional raw name
pub(crate) fn create_namespace_id(id: Option<Name>) -> NamespaceId {
    id.map(|id| {
        let qualified_id = id.qualify_with(None);
        NamespaceId {
            items: qualified_id
                .namespace_components()
                .chain(std::iter::once(qualified_id.basename()))
                .cloned()
                .collect(),
        }
    })
    .unwrap_or_default()
}

/// Converts record types from Cedar JSON schema
pub(crate) fn convert_record_type(record_type: &json_schema::RecordType<RawName>) -> CodegenResult<CedarType> {
    let fields = record_type
        .attributes
        .iter()
        .map(|(key, value)| {
            let field = CedarTypeStructField {
                optional: !value.required,
                ty: convert_cedar_to_rust(&value.ty)?,
            };
            Ok((key.to_string(), field))
        })
        .collect::<CodegenResult<_>>()?;

    Ok(CedarType::Record {
        fields,
        allows_additional: record_type.additional_attributes,
    })
}

/// Converts Cedar JSON schema types to internal CedarType representation
pub(crate) fn convert_cedar_to_rust(ty: &json_schema::Type<RawName>) -> CodegenResult<CedarType> {
    match ty {
        json_schema::Type::CommonTypeRef { type_name, .. } => Ok(CedarType::Reference(type_name.clone().into())),
        json_schema::Type::Type { ty, .. } => convert_type_variant(ty),
    }
}

/// Converts Cedar type variants to internal representation
pub(crate) fn convert_type_variant(ty: &json_schema::TypeVariant<RawName>) -> CodegenResult<CedarType> {
    match ty {
        json_schema::TypeVariant::Boolean => Ok(CedarType::Bool),
        json_schema::TypeVariant::String => Ok(CedarType::String),
        json_schema::TypeVariant::Long => Ok(CedarType::Long),
        json_schema::TypeVariant::Entity { name } => Ok(CedarType::Reference(name.clone().into())),
        json_schema::TypeVariant::EntityOrCommon { type_name } => Ok(CedarRef::from(type_name.clone()).into_cedar_ty()),
        json_schema::TypeVariant::Extension { name } => Err(CodegenError::Unsupported(format!("extention type: {name}"))),
        json_schema::TypeVariant::Record(record_type) => convert_record_type(record_type),
        json_schema::TypeVariant::Set { element } => Ok(CedarType::Set(Box::new(convert_cedar_to_rust(element.as_ref())?))),
    }
}

pub(crate) fn sanitize_identifier(s: impl AsRef<str>) -> String {
    let ident = s.as_ref();
    match ident {
        // 2015 strict keywords
        "as" | "break" | "const" | "continue" | "else" | "enum" | "false" | "fn" | "for"
        | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut" | "pub"
        | "ref" | "return" | "static" | "struct" | "trait" | "true" | "type" | "unsafe"
        | "use" | "where" | "while"
        // 2018 strict keywords
        | "dyn"
        // 2015 reserved keywords
        | "abstract" | "become" | "box" | "do" | "final" | "macro" | "override" | "priv"
        | "typeof" | "unsized" | "virtual" | "yield"
        // 2018 reserved keywords
        | "async" | "await" | "try"
        // 2024 reserved keywords
        | "gen" => format!("r#{ident}"),
        // Keywords not supported as raw identifiers
        "_" | "super" | "self" | "Self" | "extern" | "crate" => format!("{ident}_"),
        // Identifiers starting with numbers
        s if s.starts_with(|c: char| c.is_numeric()) => format!("_{ident}"),
        _ => ident.to_string(),
    }
}

/// Converts an identifier to snake_case
pub(crate) fn to_snake_ident(s: impl AsRef<str>) -> syn::Ident {
    syn::Ident::new(
        &sanitize_identifier(s.as_ref().to_snake_case()),
        proc_macro2::Span::call_site(),
    )
}

/// Converts an identifier to UpperCamelCase
pub(crate) fn to_upper_camel_ident(s: impl AsRef<str>) -> syn::Ident {
    syn::Ident::new(
        &sanitize_identifier(s.as_ref().to_upper_camel_case()),
        proc_macro2::Span::call_site(),
    )
}

/// Finds the relative path between two locations
pub(crate) fn find_relative_path(location: &[syn::Ident], dest: &[syn::Ident]) -> syn::Type {
    let common_len = location.iter().zip(dest.iter()).take_while(|(a, b)| a == b).count();

    let levels_up = location.len().saturating_sub(common_len);
    let mut path_parts = Vec::new();
    let super_ident = syn::Ident::new("super", proc_macro2::Span::call_site());

    // Add super:: for each level up
    for _ in 0..levels_up {
        path_parts.push(super_ident.clone());
    }

    // Add remaining destination path parts
    path_parts.extend_from_slice(&dest[common_len..]);

    if path_parts.is_empty() {
        panic!("Invalid path calculation");
    } else {
        syn::parse_quote!(#(#path_parts)::*)
    }
}

pub(crate) fn process_fragment(fragment: &Fragment<RawName>, config: &Config) -> CodegenResult<syn::File> {
    let mut codegen = Codegen::new(config);

    for (id, ns) in &fragment.0 {
        let mut namespace = CedarNamespace::default();

        // Process common types
        for (id, ty) in &ns.common_types {
            namespace.handle_common_type(id, ty)?;
        }

        // Process entity types
        for (id, ty) in &ns.entity_types {
            namespace.handle_entity_type(id, ty)?;
        }

        // Process actions
        for (action, ty) in &ns.actions {
            namespace.handle_action(action, ty)?;
        }

        let namespace_id = create_namespace_id(id.clone());
        codegen.add_namespace(namespace_id, namespace);
    }

    codegen.generate()
}
