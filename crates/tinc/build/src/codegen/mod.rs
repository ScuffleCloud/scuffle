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

fn strip_last_path_part(s: &str) -> &str {
    let mut parts = s.rsplitn(2, '.');
    parts.next();
    parts.next().unwrap_or(s)
}

fn get_common_import_path(package: &str, end: &str) -> syn::Path {
    let start_parts: Vec<&str> = package.split('.').collect();
    let mut end_parts: Vec<&str> = end.split('.').collect();

    let end_part = type_ident_from_str(end_parts.pop().expect("end path must not be empty")).to_string();

    let common_len = start_parts.iter().zip(&end_parts).take_while(|(a, b)| a == b).count();

    let num_supers = start_parts.len().saturating_sub(common_len);

    let mut path_parts = Vec::new();

    for _ in 0..num_supers {
        path_parts.push("super".to_string());
    }

    for end_part in end_parts[common_len..].iter() {
        path_parts.push(field_ident_from_str(end_part).to_string());
    }

    path_parts.push(end_part);

    syn::parse_str(&path_parts.join("::")).expect("failed to parse path")
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_get_common_import_path() {
        assert_eq!(get_common_import_path("a.b.c", "a.b.d"), syn::parse_str("super::D").unwrap());
        assert_eq!(get_common_import_path("a.b.c", "a.b.c.d"), syn::parse_str("D").unwrap());
        assert_eq!(get_common_import_path("a.b.c", "a.b.c"), syn::parse_str("super::C").unwrap());
        assert_eq!(
            get_common_import_path("a.b.c", "a.b"),
            syn::parse_str("super::super::B").unwrap()
        );
    }
}
