use syn::Ident;

use super::prost_sanatize;

pub(crate) fn field_ident_from_str(s: impl AsRef<str>) -> Ident {
    syn::parse_str(&prost_sanatize::to_snake(s.as_ref())).unwrap()
}

pub(crate) fn type_ident_from_str(s: impl AsRef<str>) -> Ident {
    syn::parse_str(&prost_sanatize::to_upper_camel(s.as_ref())).unwrap()
}

pub(crate) fn get_common_import_path(package: &str, end: &str) -> syn::Path {
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
