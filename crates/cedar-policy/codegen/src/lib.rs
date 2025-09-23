//! Cedar is a policy language used to express permisisons using a relationship model.
//!
//! This crate extends the [`cedar-policy`](https://docs.rs/cedar-policy) crate by adding code generator for cedar schemas.
//!
//!
//! You can then use this in combo with cedar to have type-safe schema evaluation.
//!
//! ## Example
//!
//! ```rust
//! # fn inner() {
//! let schema = std::fs::read_to_string("./static.cedarschema").expect("failed to read");
//!
//! let config = scuffle_cedar_policy_codegen::Config::default()
//!     .generate_from_schema(&schema)
//!     .expect("valid schema");
//!
//! let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("no such env")).join("generated.rs");
//! std::fs::write(output, config.to_string()).expect("failed to write output");
//! # }
//! ```
//!
//! ## License
//!
//! This project is licensed under the MIT or Apache-2.0 license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::multiple_unsafe_ops_per_block)]

mod cedar_action;
mod cedar_namespace;
mod codegen;
mod error;
mod module;
mod types;
mod utils;

use cedar_policy_core::extensions::Extensions;
use cedar_policy_core::validator::RawName;
use cedar_policy_core::validator::cedar_schema::SchemaWarning;
use cedar_policy_core::validator::json_schema::Fragment;
pub use error::{CodegenError, CodegenResult};

use crate::utils::process_fragment;

/// A config for the code generator.
pub struct Config {
    pub(crate) crate_path: syn::Path,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            crate_path: syn::parse_quote!(::scuffle_cedar_policy),
        }
    }
}

/// The output from the code generator.
pub struct CodegenOutput {
    file: syn::File,
    warnings: Vec<SchemaWarning>,
}

impl CodegenOutput {
    /// Get warnings produced from the parser
    pub fn warnings(&self) -> &[SchemaWarning] {
        &self.warnings
    }
}

impl quote::ToTokens for CodegenOutput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.file.to_tokens(tokens);
    }
}

impl std::fmt::Display for CodegenOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        prettyplease::unparse(&self.file).fmt(f)
    }
}

impl Config {
    /// Create a new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Provide a different path to find the `scuffle_cedar_policy` crate.
    pub fn crate_path(&mut self, path: syn::Path) -> &mut Self {
        self.crate_path = path;
        self
    }

    /// Generate code from a given schema string.
    pub fn generate_from_schema(&self, schema: &str) -> CodegenResult<CodegenOutput> {
        let (fragment, warnings) = Fragment::from_cedarschema_str(schema, Extensions::all_available())?;
        self.generate_from_fragment(&fragment).map(|mut out| {
            out.warnings.extend(warnings);
            out
        })
    }

    /// Generate code from a given json string.
    pub fn generate_from_json(&self, schema_json: &str) -> CodegenResult<CodegenOutput> {
        let fragment = Fragment::from_json_str(schema_json)?;
        self.generate_from_fragment(&fragment)
    }

    /// Generate code from a fragment.
    pub fn generate_from_fragment(&self, fragment: &Fragment<RawName>) -> CodegenResult<CodegenOutput> {
        let file = process_fragment(fragment, self)?;

        Ok(CodegenOutput {
            file,
            warnings: Vec::new(),
        })
    }
}
