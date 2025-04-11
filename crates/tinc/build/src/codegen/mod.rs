use std::collections::BTreeMap;

use syn::Ident;

use self::serde::{handle_enum, handle_message};
use self::service::handle_service;
use crate::extensions::Extensions;

mod prost_sanatize;
mod serde;
mod service;

fn ident_from_str(s: impl AsRef<str>) -> Ident {
    Ident::new(s.as_ref(), proc_macro2::Span::call_site())
}

fn field_ident_from_str(s: impl AsRef<str>) -> Ident {
    Ident::new(&prost_sanatize::to_snake(s.as_ref()), proc_macro2::Span::call_site())
}

fn type_ident_from_str(s: impl AsRef<str>) -> Ident {
    Ident::new(&prost_sanatize::to_upper_camel(s.as_ref()), proc_macro2::Span::call_site())
}

fn get_common_import_path(start: &str, end: &str) -> syn::Path {
    let start_parts: Vec<&str> = start.split('.').collect();
    let end_parts: Vec<&str> = end.split('.').collect();
    let common_len = start_parts.iter().zip(&end_parts).take_while(|(a, b)| a == b).count();
    let num_supers = start_parts.len().saturating_sub(common_len + 2);
    let super_prefix = "super::".repeat(num_supers);
    let mut parts = end_parts[common_len..]
        .iter()
        .copied()
        .map(|part| part.to_owned())
        .collect::<Vec<_>>();
    if let Some(last) = parts.last_mut() {
        *last = type_ident_from_str(&last).to_string();
    }
    let relative_path = parts.join("::");

    syn::parse_str(&format!("{}{}", super_prefix, relative_path)).expect("failed to parse path")
}

pub fn generate_modules(
    extensions: &Extensions,
    prost: &mut tonic_build::Config,
) -> anyhow::Result<BTreeMap<String, Vec<syn::Item>>> {
    let mut modules = BTreeMap::new();

    extensions
        .messages()
        .iter()
        .try_for_each(|(key, message)| handle_message(key, message, prost, &mut modules))?;

    extensions
        .enums()
        .iter()
        .try_for_each(|(key, enum_)| handle_enum(key, enum_, prost, &mut modules))?;

    extensions
        .services()
        .iter()
        .try_for_each(|(key, service)| handle_service(key, service, extensions, prost, &mut modules))?;

    Ok(modules)
}
