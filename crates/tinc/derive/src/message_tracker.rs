use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

struct TincContainerOptions {
    pub crate_path: syn::Path,
    pub tagged: bool,
    pub with_non_finite_values: bool,
}

impl TincContainerOptions {
    fn from_attributes<'a>(attrs: impl IntoIterator<Item = &'a syn::Attribute>) -> syn::Result<Self> {
        let mut crate_ = None;
        let mut tagged = false;
        let mut with_non_finite_values = false;

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
                    } else if meta.path.is_ident("tagged") {
                        tagged = true;
                    } else if meta.path.is_ident("with_non_finite_values") {
                        with_non_finite_values = true;
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

        if tagged {
            options.tagged = true;
        }
        if with_non_finite_values {
            options.with_non_finite_values = true;
        }

        Ok(options)
    }
}

impl Default for TincContainerOptions {
    fn default() -> Self {
        Self {
            crate_path: syn::parse_str("::tinc").unwrap(),
            tagged: false,
            with_non_finite_values: false,
        }
    }
}

#[derive(Default)]
struct TincFieldOptions {
    pub enum_path: Option<syn::Path>,
    pub oneof: bool,
    pub with_non_finite_values: bool,
}

impl TincFieldOptions {
    fn from_attributes<'a>(attrs: impl IntoIterator<Item = &'a syn::Attribute>) -> syn::Result<Self> {
        let mut enum_ = None;
        let mut oneof = false;
        let mut with_non_finite_values = false;

        for attr in attrs {
            let syn::Meta::List(list) = &attr.meta else {
                continue;
            };

            if list.path.is_ident("tinc") {
                list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("enum") {
                        if enum_.is_some() {
                            return Err(meta.error("enum option already set"));
                        }

                        let _: syn::token::Eq = meta.input.parse()?;
                        let path: syn::LitStr = meta.input.parse()?;
                        enum_ = Some(syn::parse_str(&path.value())?);
                    } else if meta.path.is_ident("oneof") {
                        oneof = true;
                    } else if meta.path.is_ident("with_non_finite_values") {
                        with_non_finite_values = true;
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

        if oneof {
            options.oneof = true;
        }
        if with_non_finite_values {
            options.with_non_finite_values = true;
        }

        Ok(options)
    }
}

pub(crate) fn derive_message_tracker(input: TokenStream) -> TokenStream {
    let input = match syn::parse2::<syn::DeriveInput>(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error(),
    };

    let opts = match TincContainerOptions::from_attributes(&input.attrs) {
        Ok(options) => options,
        Err(e) => return e.to_compile_error(),
    };

    match &input.data {
        syn::Data::Struct(data) => derive_message_tracker_struct(input.ident, opts, data),
        syn::Data::Enum(data) => derive_message_tracker_enum(input.ident, opts, data),
        _ => syn::Error::new(input.span(), "Tracker can only be derived for structs or enums").into_compile_error(),
    }
}

fn derive_message_tracker_struct(ident: syn::Ident, opts: TincContainerOptions, data: &syn::DataStruct) -> TokenStream {
    let TincContainerOptions {
        crate_path,
        tagged,
        with_non_finite_values,
    } = opts;
    if tagged {
        return syn::Error::new(ident.span(), "tagged can only be used on enums").into_compile_error();
    }
    if with_non_finite_values {
        return syn::Error::new(ident.span(), "with_non_finite_values can only be used on floats").into_compile_error();
    }

    let syn::Fields::Named(fields) = &data.fields else {
        return syn::Error::new(ident.span(), "Tracker can only be derived for structs with named fields")
            .into_compile_error();
    };

    let tracker_ident = syn::Ident::new(&format!("{ident}Tracker"), ident.span());

    let struct_fields = fields
        .named
        .iter()
        .map(|f| {
            let field_ident = f.ident.as_ref().expect("field must have an identifier");
            let ty = &f.ty;

            let TincFieldOptions {
                enum_path,
                oneof,
                with_non_finite_values,
            } = TincFieldOptions::from_attributes(&f.attrs)?;

            if enum_path.is_some() && (oneof || with_non_finite_values) {
                return Err(syn::Error::new(
                    f.span(),
                    "enum cannot be set with oneof or with_non_finite_values",
                ));
            }
            if oneof && with_non_finite_values {
                return Err(syn::Error::new(f.span(), "oneof cannot be set with with_non_finite_values"));
            }

            let ty = match enum_path {
                Some(enum_path) => quote! { <#ty as #crate_path::__private::EnumHelper>::Target<#enum_path> },
                None if oneof => quote! { <#ty as #crate_path::__private::OneOfHelper>::Target },
                None if with_non_finite_values => {
                    quote! { <#ty as #crate_path::__private::FloatWithNonFinDesHelper>::Target }
                }
                None => quote! { #ty },
            };

            Ok(quote! {
                pub #field_ident: Option<<#ty as #crate_path::__private::TrackerFor>::Tracker>
            })
        })
        .collect::<syn::Result<Vec<_>>>();

    let struct_fields = match struct_fields {
        Ok(fields) => fields,
        Err(e) => return e.to_compile_error(),
    };

    quote! {
        #[allow(clippy::all, dead_code, unused_imports, unused_variables, unused_parens)]
        const _: () = {
            #[derive(Debug, Default)]
            pub struct #tracker_ident {
                #(#struct_fields),*
            }

            impl #crate_path::__private::Tracker for #tracker_ident {
                type Target = #ident;

                #[inline(always)]
                fn allow_duplicates(&self) -> bool {
                    true
                }
            }

            impl #crate_path::__private::TrackerFor for #ident {
                type Tracker = #crate_path::__private::StructTracker<#tracker_ident>;
            }
        };
    }
}

fn derive_message_tracker_enum(ident: syn::Ident, opts: TincContainerOptions, data: &syn::DataEnum) -> TokenStream {
    let TincContainerOptions {
        crate_path,
        tagged,
        with_non_finite_values,
    } = opts;
    let tracker_ident = syn::Ident::new(&format!("{ident}Tracker"), ident.span());

    if with_non_finite_values {
        return syn::Error::new(ident.span(), "with_non_finite_values can only be used on floats").into_compile_error();
    }

    let variants = data
        .variants
        .iter()
        .map(|v| {
            let variant_ident = &v.ident;
            let syn::Fields::Unnamed(unnamed) = &v.fields else {
                return Err(syn::Error::new(
                    v.span(),
                    "Tracker can only be derived for enums with unnamed variants",
                ));
            };

            if unnamed.unnamed.len() != 1 {
                return Err(syn::Error::new(
                    v.span(),
                    "Tracker can only be derived for enums with a single field variants",
                ));
            }

            let field = &unnamed.unnamed[0];
            let ty = &field.ty;

            let TincFieldOptions {
                enum_path,
                oneof,
                with_non_finite_values,
            } = TincFieldOptions::from_attributes(v.attrs.iter().chain(field.attrs.iter()))?;

            if oneof {
                return Err(syn::Error::new(
                    v.span(),
                    "oneof can only be used on struct fields, not on enum variants",
                ));
            }

            let ty = match enum_path {
                Some(enum_path) => quote! {
                    <#ty as #crate_path::__private::EnumHelper>::Target<#enum_path>
                },
                None if with_non_finite_values => quote! {
                    <#ty as #crate_path::__private::FloatWithNonFinDesHelper>::Target
                },
                None => quote! {
                    #ty
                },
            };

            Ok((
                quote! {
                    #variant_ident(<#ty as #crate_path::__private::TrackerFor>::Tracker)
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

    let tracker = if tagged {
        quote! {
            #crate_path::__private::TaggedOneOfTracker<#tracker_ident>
        }
    } else {
        quote! {
            #crate_path::__private::OneOfTracker<#tracker_ident>
        }
    };

    quote! {
        #[allow(clippy::all, dead_code, unused_imports, unused_variables, unused_parens)]
        const _: () = {
            #[derive(std::fmt::Debug)]
            pub enum #tracker_ident {
                #(#variants),*
            }

            impl #crate_path::__private::Tracker for #tracker_ident {
                type Target = #ident;

                #[inline(always)]
                fn allow_duplicates(&self) -> bool {
                    match self {
                        #(Self::#variant_idents(v) => v.allow_duplicates()),*
                    }
                }
            }

            impl #crate_path::__private::TrackerFor for #ident {
                type Tracker = #tracker;
            }
        };
    }
}
