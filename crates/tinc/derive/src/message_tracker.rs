use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::spanned::Spanned;

struct TincContainerOptions {
    pub crate_path: syn::Path,
}

impl TincContainerOptions {
    pub fn from_attributes<'a>(attrs: impl IntoIterator<Item = &'a syn::Attribute>) -> syn::Result<Self> {
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

        let mut options = TincContainerOptions::default();
        if let Some(crate_) = crate_ {
            options.crate_path = crate_;
        }

        Ok(options)
    }
}

impl Default for TincContainerOptions {
    fn default() -> Self {
        Self {
            crate_path: syn::parse_str("::tinc").unwrap(),
        }
    }
}

#[derive(Default)]
struct TincFieldOptions {
    pub enum_path: Option<syn::Path>,
    pub is_one_of: bool,
}

impl TincFieldOptions {
    pub fn from_attributes<'a>(attrs: impl IntoIterator<Item = &'a syn::Attribute>) -> syn::Result<Self> {
        let mut enum_ = None;
        let mut is_one_of = false;

        for attr in attrs {
            let syn::Meta::List(list) = &attr.meta else {
                continue;
            };

            if list.path.is_ident("tinc") {
                list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("enum") {
                        let _: syn::token::Eq = meta.input.parse()?;
                        let path: syn::LitStr = meta.input.parse()?;
                        enum_ = Some(syn::parse2(path.to_token_stream())?);
                    } else if meta.path.is_ident("oneof") {
                        is_one_of = true;
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

        options.is_one_of = is_one_of;

        Ok(options)
    }
}

pub fn derive_message_tracker(input: TokenStream) -> TokenStream {
    let input = match syn::parse2::<syn::DeriveInput>(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error(),
    };

    let TincContainerOptions { crate_path } = match TincContainerOptions::from_attributes(&input.attrs) {
        Ok(options) => options,
        Err(e) => return e.to_compile_error(),
    };

    match &input.data {
        syn::Data::Struct(data) => derive_message_tracker_struct(input.ident, crate_path, data),
        syn::Data::Enum(data) => derive_message_tracker_enum(input.ident, crate_path, data),
        _ => syn::Error::new(input.span(), "MessageTracker can only be derived for structs or enums").into_compile_error(),
    }
}

fn derive_message_tracker_struct(ident: syn::Ident, crate_path: syn::Path, data: &syn::DataStruct) -> TokenStream {
    let syn::Fields::Named(fields) = &data.fields else {
        return syn::Error::new(
            ident.span(),
            "MessageTracker can only be derived for structs with named fields",
        )
        .into_compile_error();
    };

    let tracker_ident = syn::Ident::new(&format!("{ident}Tracker"), ident.span());
    let struct_fields = fields
        .named
        .iter()
        .map(|f| {
            let field_ident = f.ident.as_ref().expect("field must have an identifier");
            let ty = &f.ty;

            let TincFieldOptions { enum_path, is_one_of } = TincFieldOptions::from_attributes(&f.attrs)?;

            let ty = match (enum_path, is_one_of) {
                (Some(enum_path), false) => {
                    quote! {
                        <#ty as #crate_path::__private::de::EnumHelper>::Target<#enum_path>
                    }
                }
                (None, true) => {
                    quote! {
                        <#ty as #crate_path::__private::de::OneOfHelper>::Target
                    }
                }
                (None, false) => {
                    quote! {
                        #ty
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        f.span(),
                        "only one of enum or one_of can be specified for a field",
                    ));
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
                type Target = #ident;

                #[inline(always)]
                fn allow_duplicates(&self) -> bool {
                    true
                }
            }

            impl #crate_path::__private::de::TrackerFor for #ident {
                type Tracker = #crate_path::__private::de::MessageTracker<#tracker_ident>;
            }
        };
    }
}

fn derive_message_tracker_enum(ident: syn::Ident, crate_path: syn::Path, data: &syn::DataEnum) -> TokenStream {
    let tracker_ident = syn::Ident::new(&format!("{ident}Tracker"), ident.span());

    let variants = data
        .variants
        .iter()
        .map(|v| {
            let variant_ident = &v.ident;
            let syn::Fields::Unnamed(unnamed) = &v.fields else {
                return Err(syn::Error::new(
                    v.span(),
                    "MessageTracker can only be derived for enums with unnamed variants",
                ));
            };

            if unnamed.unnamed.len() != 1 {
                return Err(syn::Error::new(
                    v.span(),
                    "MessageTracker can only be derived for enums with a single field variants",
                ));
            }

            let field = &unnamed.unnamed[0];
            let ty = &field.ty;

            let TincFieldOptions { enum_path, is_one_of } =
                TincFieldOptions::from_attributes(v.attrs.iter().chain(field.attrs.iter()))?;
            if is_one_of {
                return Err(syn::Error::new(v.span(), "one_of is not supported for enum variants"));
            }

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

            Ok((
                quote! {
                    #variant_ident(<#ty as #crate_path::__private::de::TrackerFor>::Tracker)
                },
                quote! {
                    #variant_ident
                },
            ))
        })
        .collect::<syn::Result<(Vec<_>, Vec<_>)>>();

    let (variants, variant_idents) = match variants {
        Ok(variants) => variants,
        Err(e) => return e.to_compile_error(),
    };

    quote! {
        const _: () = {
            #[derive(Debug)]
            pub enum #tracker_ident {
                #(#variants),*
            }

            impl #crate_path::__private::de::Tracker for #tracker_ident {
                type Target = #ident;

                #[inline(always)]
                fn allow_duplicates(&self) -> bool {
                    match self {
                        #(Self::#variant_idents(v) => v.allow_duplicates()),*
                    }
                }
            }

            impl #crate_path::__private::de::TrackerFor for #ident {
                type Tracker = #tracker_ident;
            }
        };
    }
}
