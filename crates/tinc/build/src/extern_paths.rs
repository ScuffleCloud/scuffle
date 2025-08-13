use std::collections::BTreeMap;

use quote::ToTokens;
use syn::parse_quote;

use crate::Mode;
use crate::codegen::prost_sanatize::to_upper_camel;
use crate::codegen::utils::{field_ident_from_str, type_ident_from_str};
use crate::types::ProtoPath;

#[derive(Clone)]
pub(crate) struct ExternPaths {
    paths: BTreeMap<ProtoPath, syn::Path>,
}

impl std::fmt::Debug for ExternPaths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();

        for (key, value) in &self.paths {
            map.key(&key.as_ref());
            map.value(&value.to_token_stream().to_string());
        }

        Ok(())
    }
}

impl ExternPaths {
    pub(crate) fn new(mode: Mode) -> Self {
        let mut paths = BTreeMap::new();

        paths.insert(ProtoPath::new("google.protobuf"), parse_quote!(::tinc::well_known::#mode));
        paths.insert(ProtoPath::new("tinc"), parse_quote!(::tinc::well_known::#mode::tinc));

        Self { paths }
    }

    pub(crate) fn resolve(&self, path: &str) -> Option<syn::Path> {
        if let Some(path) = self.paths.get(path) {
            return Some(path.clone());
        }

        for (idx, _) in path.rmatch_indices('.') {
            if let Some(rust_path) = self.paths.get(&path[..idx]) {
                let mut segments = path[idx + 1..].split('.');
                let ident_type = segments.next_back().map(to_upper_camel).map(type_ident_from_str);
                let segments = segments.map(field_ident_from_str).chain(ident_type);

                return Some(parse_quote!(
                    #rust_path::#(#segments)::*
                ));
            }
        }

        None
    }

    pub(crate) fn contains(&self, path: &str) -> bool {
        if self.paths.contains_key(path) {
            return true;
        }

        for (idx, _) in path.rmatch_indices('.') {
            if self.paths.contains_key(&path[..idx]) {
                return true;
            }
        }

        false
    }

    pub(crate) fn paths(&self) -> std::collections::btree_map::Iter<'_, ProtoPath, syn::Path> {
        self.paths.iter()
    }
}
