#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

use darling::FromDeriveInput;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(IsoBox, attributes(iso_box))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input);

    match box_impl(derive_input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[derive(Debug, darling::FromDeriveInput)]
#[darling(attributes(iso_box), supports(struct_named))]
struct IsoBoxOpts {
    ident: syn::Ident,
    data: darling::ast::Data<(), IsoBoxField>,
    box_type: syn::LitByteStr,
    #[darling(default)]
    full_header: bool,
    #[darling(default = "default_crate_path")]
    crate_path: syn::Path,
}

fn default_crate_path() -> syn::Path {
    syn::parse_str("::isobmff").unwrap()
}

fn into_fields_checked(
    data: darling::ast::Data<(), IsoBoxField>,
    full_header: bool,
) -> syn::Result<darling::ast::Fields<IsoBoxField>> {
    let fields = data.take_struct().expect("unreachable: only structs supported");

    if let Some(field) = fields.iter().find(|f| {
        [f.header, f.full_header, f.nested_box, f.from.is_some()]
            .iter()
            .map(|b| *b as u8)
            .sum::<u8>()
            > 1
    }) {
        return Err(syn::Error::new_spanned(
            field.ident.as_ref().expect("unreachable: only named fields supported"),
            "Only one of header, full_header, nested_box or from can be used",
        ));
    }

    if let Some(field) = fields.iter().find(|f| f.remaining && (f.header || f.full_header)) {
        return Err(syn::Error::new_spanned(
            field.ident.as_ref().expect("unreachable: only named fields supported"),
            "remaining field cannot be used with header or full_header",
        ));
    }

    if let Some(field) = fields.iter().filter(|f| f.remaining).nth(1) {
        return Err(syn::Error::new_spanned(
            field.ident.as_ref().expect("unreachable: only named fields supported"),
            "Only one field can be marked as remaining",
        ));
    }

    if let Some(field) = fields.iter().filter(|f| f.header).nth(1) {
        return Err(syn::Error::new_spanned(
            field.ident.as_ref().expect("unreachable: only named fields supported"),
            "Only one field can be marked as header",
        ));
    }

    if let Some(field) = fields.iter().filter(|f| f.full_header).nth(1) {
        return Err(syn::Error::new_spanned(
            field.ident.as_ref().expect("unreachable: only named fields supported"),
            "Only one field can be marked as full_header",
        ));
    }

    if !full_header {
        if let Some(field) = fields.iter().find(|f| f.full_header) {
            return Err(syn::Error::new_spanned(
                field.ident.as_ref().expect("unreachable: only named fields supported"),
                "full_header field can only be used with full_header attribute",
            ));
        }
    }

    Ok(fields)
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(iso_box))]
struct IsoBoxField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    from: Option<syn::Type>,
    #[darling(default)]
    remaining: bool,
    #[darling(default)]
    header: bool,
    #[darling(default)]
    full_header: bool,
    #[darling(default)]
    nested_box: bool,
}

fn box_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let opts = IsoBoxOpts::from_derive_input(&input)?;
    let crate_path = opts.crate_path;

    let mut field_parsers = Vec::new();
    let mut constructor_fields = Vec::new();

    let fields = into_fields_checked(opts.data, opts.full_header)?;

    let header_constructor = fields.iter().find(|f| f.header).map(|f| {
        let name = f.ident.as_ref().expect("unreachable: only named fields supported");
        quote! {
            #name: header,
        }
    });

    if opts.full_header {
        let full_header_field_name = fields
            .iter()
            .find(|f| f.full_header)
            .map(|f| f.ident.as_ref().expect("unreachable: only named fields supported"));

        if let Some(full_header_field_name) = full_header_field_name {
            field_parsers.push(quote! {
                let #full_header_field_name = #crate_path::FullBoxHeader::demux(&mut payload_reader)?;
            });
            constructor_fields.push(full_header_field_name.clone());
        } else {
            field_parsers.push(quote! {
                #crate_path::FullBoxHeader::demux(&mut payload_reader)?;
            });
        }
    }

    for field in fields.into_iter().filter(|f| !f.full_header && !f.header) {
        let field_name = field.ident.clone().expect("unreachable: only named fields supported");

        let read_field = if field.remaining {
            read_field_remaining(field, &crate_path)
        } else if field.from.is_some() {
            read_field_with_from(field, &crate_path)
        } else {
            read_field(field, &crate_path)
        };

        field_parsers.push(quote! {
            let #field_name = #read_field;
        });

        constructor_fields.push(field_name);
    }

    let box_type = opts.box_type;
    let ident = opts.ident;

    let output = quote! {
        impl<'a> IsoBox<'a> for #ident {
            const TYPE: [u8; 4] = *#box_type;

            fn demux<R: ::scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>>(header: #crate_path::BoxHeader, mut payload_reader: R) -> ::std::io::Result<Self> {
                use #crate_path::read_field::{i24, i48, u24, u48};
                #(#field_parsers)*
                Ok(Self {
                    #header_constructor
                    #(#constructor_fields,)*
                })
            }
        }
    };

    Ok(output)
}

fn read_field_remaining(field: IsoBoxField, crate_path: &syn::Path) -> proc_macro2::TokenStream {
    let field_type = field.ty;

    if let Some(from) = field.from {
        quote! {
            ::std::iter::Iterator::collect::<::std::vec::Vec<_>>(
                ::std::iter::Iterator::map(
                    ::std::iter::IntoIterator::into_iter(<::std::vec::Vec<#from> as #crate_path::read_field::ReadRemaining>::read_remaining(&mut payload_reader, None)?),
                    ::std::convert::From::from,
                )
            )
        }
    } else {
        quote! {
            <#field_type as #crate_path::read_field::ReadRemaining>::read_remaining(&mut payload_reader, None)?
        }
    }
}

fn read_field_with_from(field: IsoBoxField, crate_path: &syn::Path) -> proc_macro2::TokenStream {
    let field_type = field.ty.clone();
    let read_field = read_field(field, crate_path);

    quote! {
        <#field_type as ::std::convert::From<_>>::from(#read_field)
    }
}

fn read_field(field: IsoBoxField, crate_path: &syn::Path) -> proc_macro2::TokenStream {
    let field_type = field.from.unwrap_or(field.ty);

    quote! {
        <#field_type as #crate_path::read_field::ReadField>::read_field(&mut payload_reader)?
    }
}
