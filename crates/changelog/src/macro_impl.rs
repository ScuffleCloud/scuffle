use std::fmt::{Display, Write};

use convert_case::Casing;
use proc_macro2::{Span, TokenStream};
use quote::quote;

#[derive(Debug)]
struct ChangeLogEntry<'a> {
    module_name: syn::Ident,
    module_comment: String,
    lines: Vec<&'a str>,
}

static HEADING_REGEX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"^## \[([^]]+)\](?:\(([^)]+)\))?(?: - (\d{4}-\d{2}-\d{2}))?$").unwrap());

fn parse_changelog(changelog: &str) -> Result<Vec<ChangeLogEntry<'_>>, String> {
    let mut entries = Vec::new();
    let mut lines_iter = changelog.lines().peekable();
    while let Some(line) = lines_iter.next() {
        let Some(capture) = HEADING_REGEX.captures(line) else {
            continue;
        };

        let name = capture.get(1).unwrap().as_str();
        let url = capture.get(2).map(|m| m.as_str());
        let date = capture.get(3).map(|m| m.as_str());
        let semver = semver::Version::parse(name).ok();
        let module_name = if let Some(version) = &semver {
            let mut name = format!("v{}_{}_{}", version.major, version.minor, version.patch);
            if !version.pre.is_empty() {
                name.push('_');
                name.extend(version.pre.chars().map(|c| match c {
                    '-' | '.' => '_',
                    c => c,
                }));
            }
            name
        } else {
            name.to_case(convert_case::Case::Snake)
        };

        let mut lines = Vec::new();
        while let Some(line) = lines_iter.peek() {
            if HEADING_REGEX.is_match(line) {
                break;
            }

            lines.push(lines_iter.next().unwrap());
        }

        if lines.iter().any(|line| !line.trim().is_empty()) {
            entries.push(ChangeLogEntry {
                module_name: syn::Ident::new(&module_name, Span::call_site()),
                lines,
                module_comment: fmtools::fmt(|f| {
                    if let Some(semver) = &semver {
                        f.write_str("Release ")?;
                        if url.is_some() {
                            f.write_char('[')?;
                        }
                        semver.fmt(f)?;
                        if let Some(url) = url {
                            f.write_char(']')?;
                            f.write_char('(')?;
                            f.write_str(url)?;
                            f.write_char(')')?;
                        }
                    } else {
                        f.write_str(name)?;
                    }

                    if let Some(date) = &date {
                        f.write_str(" (")?;
                        f.write_str(date)?;
                        f.write_char(')')?;
                    }

                    Ok(())
                })
                .to_string(),
            });
        }
    }

    Ok(entries)
}

pub(crate) fn changelog(_: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let syn::ItemMod {
        attrs,
        content,
        ident,
        mod_token,
        vis,
        ..
    } = syn::parse2(item)?;

    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let path = std::path::PathBuf::from(manifest_dir);
    let changelog =
        std::fs::read_to_string(path.join("CHANGELOG.md")).map_err(|err| syn::Error::new(ident.span(), err.to_string()))?;
    let entries = parse_changelog(&changelog).map_err(|err| syn::Error::new(ident.span(), err.to_string()))?;

    let entries = entries.into_iter().map(
        |ChangeLogEntry {
             module_name,
             lines,
             module_comment,
         }| {
            quote! {
                #[doc = concat!(" ", #module_comment)]
                #( #[doc = concat!(" ", #lines)] )*
                #vis #mod_token #module_name {}
            }
        },
    );

    let content = content.unwrap_or_default().1;

    Ok(quote! {
        #(#attrs)*
        #vis #mod_token #ident {
            #(#entries)*
            #(#content)*
        }
    })
}
