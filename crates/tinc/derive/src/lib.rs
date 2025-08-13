//! Derive Macro helpers for `tinc`
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
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

use proc_macro::TokenStream;

mod message_tracker;

/// `Tracker` is used to track field presence when doing JSON deserialization.
/// This macro will generate the tracker for the given structure.
/// ## Container Opts
/// - `crate_path`: A string which is the path to the `tinc` crate, by default `::tinc`
/// - `tagged`: Can only be used on enums to denote a tagged enum, default is false.
/// ## Field / Variant Opts
/// - `enum_path`: Forces the field to be treated as an enum, default is None.
/// - `oneof`: The field should be treated as a oneof.
#[proc_macro_derive(Tracker, attributes(tinc))]
pub fn derive_message_tracker(input: TokenStream) -> TokenStream {
    message_tracker::derive_message_tracker(input.into()).into()
}
