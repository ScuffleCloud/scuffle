use proc_macro::TokenStream;

mod message_tracker;

#[proc_macro_derive(TincMessageTracker, attributes(tinc))]
pub fn derive_message_tracker(input: TokenStream) -> TokenStream {
    message_tracker::derive_message_tracker(input.into()).into()
}
