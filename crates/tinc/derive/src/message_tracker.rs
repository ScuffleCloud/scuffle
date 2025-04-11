use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

struct TincStructOptions {
    pub crate_path: syn::Path,
}

impl TincStructOptions {
    pub fn from_attributes(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut crate_ = None;
        for attr in attrs {
            let syn::Meta::List(list) = &attr.meta else {
                continue;
            };

            if list.path.is_ident("tinc") {
                list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("crate") {
                        if crate_.is_some() {
                            return Err(meta.error("crate option already set"));
                        }

                        let _: syn::token::Eq = meta.input.parse()?;
                        let path: syn::LitStr = meta.input.parse()?;
                        crate_ = Some(syn::parse_str(&path.value())?);
                    } else {
                        return Err(meta.error("unsupported attribute"));
                    }

                    Ok(())
                })?;
            }
        }

        let mut options = TincStructOptions::default();
        if let Some(crate_) = crate_ {
            options.crate_path = crate_;
        }

        Ok(options)
    }
}

impl Default for TincStructOptions {
    fn default() -> Self {
        Self {
            crate_path: syn::parse_str("::tinc").unwrap(),
        }
    }
}

#[derive(Default)]
struct TincFieldOptions {
    pub enum_path: Option<syn::Path>,
}

impl TincFieldOptions {
    pub fn from_attributes(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut enum_ = None;

        for attr in attrs {
            let syn::Meta::List(list) = &attr.meta else {
                continue;
            };

            if list.path.is_ident("tinc") {
                list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("enum") {
                        let _: syn::token::Eq = meta.input.parse()?;
                        let path: syn::LitStr = meta.input.parse()?;
                        enum_ = Some(syn::parse_str(&path.value())?);
                    } else {
                        return Err(meta.error("unsupported attribute"));
                    }

                    Ok(())
                })?;
            }
        }

        let mut options = TincFieldOptions::default();
        if let Some(enum_) = enum_ {
            options.enum_path = Some(enum_);
        }

        Ok(options)
    }
}

pub fn derive_message_tracker(input: TokenStream) -> TokenStream {
    let input = match syn::parse2::<syn::DeriveInput>(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error(),
    };

    let TincStructOptions { crate_path } = match TincStructOptions::from_attributes(&input.attrs) {
        Ok(options) => options,
        Err(e) => return e.to_compile_error(),
    };

    let syn::Data::Struct(data) = &input.data else {
        return syn::Error::new(input.span(), "MessageTracker can only be derived for structs").into_compile_error();
    };

    let syn::Fields::Named(fields) = &data.fields else {
        return syn::Error::new(
            input.span(),
            "MessageTracker can only be derived for structs with named fields",
        )
        .into_compile_error();
    };

    let struct_ident = input.ident;
    let tracker_ident = syn::Ident::new(&format!("{struct_ident}Tracker"), struct_ident.span());
    let struct_fields = fields
        .named
        .iter()
        .map(|f| {
            let field_ident = f.ident.as_ref().expect("field must have an identifier");
            let ty = &f.ty;

            let TincFieldOptions { enum_path } = TincFieldOptions::from_attributes(&f.attrs)?;

            let ty = match enum_path {
                Some(enum_path) => {
                    quote! {
                        <#ty as #crate_path::__private::de::EnumHelper>::Target<#enum_path>
                    }
                }
                None => {
                    quote! {
                        #ty
                    }
                }
            };

            Ok(quote! {
                pub #field_ident: Option<<#ty as #crate_path::__private::de::TrackerFor>::Tracker>
            })
        })
        .collect::<syn::Result<Vec<_>>>();

    let struct_fields = match struct_fields {
        Ok(fields) => fields,
        Err(e) => return e.to_compile_error(),
    };

    quote! {
        const _: () = {
            #[derive(Debug, Default)]
            pub struct #tracker_ident {
                #(#struct_fields),*
            }

            impl #crate_path::__private::de::Tracker for #tracker_ident {
                type Target = #struct_ident;

                #[inline(always)]
                fn allow_duplicates(&self) -> bool {
                    true
                }
            }

            impl #crate_path::__private::de::TrackerFor for #struct_ident {
                type Tracker = #crate_path::__private::de::MessageTracker<#tracker_ident>;
            }
        };
    }
}
