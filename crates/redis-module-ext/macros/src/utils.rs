use std::ffi::CString;

use proc_macro2::TokenStream;
use syn::Token;
use syn::parse::Parser;

pub fn repeated_parse<T: syn::parse::Parse>(item: &syn::Meta) -> darling::Result<Vec<T>> {
    match item {
        syn::Meta::List(list) => Ok(syn::punctuated::Punctuated::<T, Token![,]>::parse_terminated
            .parse2(list.tokens.clone())
            .map(|punctuated| punctuated.into_iter().collect())?),
        _ => Err(darling::Error::custom("expected list").with_span(item)),
    }
}

pub fn format_command_ident(ident: &syn::Ident) -> syn::Ident {
    quote::format_ident!("__command__{ident}").set_span(ident.span())
}

pub fn str_to_cstr(s: &syn::LitStr) -> syn::Result<syn::LitCStr> {
    let Ok(cstr) = CString::new(s.value()) else {
        return Err(syn::Error::new(s.span(), "string cannot have nul bytes"));
    };

    Ok(syn::LitCStr::new(&cstr, s.span()))
}

pub fn opt_to_tokens<T: quote::ToTokens>(opt: Option<T>) -> TokenStream {
    match opt {
        Some(opt) => quote::quote!(::core::option::Option::Some(#opt)),
        None => quote::quote!(::core::option::Option::None),
    }
}

pub fn collect_to_vec<T: quote::ToTokens>(items: impl IntoIterator<Item = T>) -> TokenStream {
    let items = items.into_iter();
    quote::quote! {
        ::std::vec::Vec::from(::std::boxed::Box::new([#(#items),*]) as ::std::boxed::Box<[_]>)
    }
}

pub trait SetSpan {
    fn set_span(&self, span: proc_macro2::Span) -> Self;
}

impl<T> SetSpan for T
where
    T: quote::ToTokens + syn::parse::Parse,
{
    fn set_span(&self, span: proc_macro2::Span) -> Self {
        syn::LitStr::new(&self.to_token_stream().to_string(), span).parse().unwrap()
    }
}
