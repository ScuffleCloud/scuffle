#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

//! Derive Macro helpers for `tinc`

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
