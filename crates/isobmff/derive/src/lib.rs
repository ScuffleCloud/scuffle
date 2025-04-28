#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

use darling::FromDeriveInput;
use quote::{ToTokens, format_ident, quote};
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

        if field.remaining {
            field_parsers.push(read_field_remaining(field, &crate_path)?);
        } else if field.from.is_some() {
            field_parsers.push(read_field_with_from(field, &crate_path, false)?);
        } else {
            field_parsers.push(read_field(field, &crate_path, false)?);
        }

        constructor_fields.push(field_name);
    }

    let box_type = opts.box_type;
    let ident = opts.ident;

    let output = quote! {
        impl<'a> IsoBox<'a> for #ident {
            const TYPE: [u8; 4] = *#box_type;

            fn demux<R: ::scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>>(header: #crate_path::BoxHeader, mut payload_reader: R) -> ::std::io::Result<Self> {
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

fn read_field_remaining(mut field: IsoBoxField, crate_path: &syn::Path) -> syn::Result<proc_macro2::TokenStream> {
    let field_name = field.ident.clone().expect("unreachable: only named fields supported");

    let syn::Type::Path(type_path) = field.ty.clone() else {
        return Err(syn::Error::new_spanned(
            field.ty,
            "Only Vec<T> is supported for remaining fields",
        ));
    };

    let segment = type_path
        .path
        .segments
        .last()
        .expect("unreachable: type path should have at least one segment");
    if segment.ident != "Vec" {
        return Err(syn::Error::new_spanned(
            segment,
            "Only Vec<T> is supported for remaining fields",
        ));
    }
    let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            segment,
            "Only Vec<T> is supported for remaining fields",
        ));
    };
    let Some(inner_type) = args.into_iter().find_map(|arg| {
        if let syn::GenericArgument::Type(ty) = arg {
            Some(ty)
        } else {
            None
        }
    }) else {
        return Err(syn::Error::new_spanned(
            segment,
            "Only Vec<T> is supported for remaining fields",
        ));
    };

    field.ty = inner_type.clone();

    let read_field = if field.from.is_some() {
        read_field_with_from(field, crate_path, true)?
    } else {
        read_field(field, crate_path, true)?
    };

    Ok(quote! {
        let #field_name = {
            let mut remaining = Vec::new();
            loop {
                let value = {
                    #read_field
                    #field_name
                };
                remaining.push(value);
            }
            remaining
        };
    })
}

fn read_field_with_from(
    field: IsoBoxField,
    crate_path: &syn::Path,
    break_on_eof: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    let field_name = field.ident.clone().expect("unreachable: only named fields supported");
    let field_type = field.ty.clone();
    let read_field = read_field(field, crate_path, break_on_eof)?;

    Ok(quote! {
        #read_field
        let #field_name: #field_type = ::std::convert::From::from(#field_name);
    })
}

const READ_FN_TYPES: [&str; 16] = [
    "f32", "f64", "i8", "i16", "i24", "i32", "i48", "i64", "i128", "u8", "u16", "u24", "u32", "u48", "u64", "u128",
];

fn read_field(field: IsoBoxField, crate_path: &syn::Path, break_on_eof: bool) -> syn::Result<proc_macro2::TokenStream> {
    let field_name = field.ident.expect("unreachable: only named fields supported");
    let field_type = field.from.unwrap_or(field.ty);
    let field_type_str = field_type.to_token_stream().to_string();

    if field.nested_box {
        if break_on_eof {
            Ok(quote! {
                let #field_name = {
                    let box_header = match #crate_path::BoxHeader::demux(&mut payload_reader) {
                        Ok(v) => v,
                        Err(e) if e.kind() == ::std::io::ErrorKind::UnexpectedEof => {
                            break;
                        },
                        Err(e) => return Err(e),
                    };

                    let res = if let Some(size) = box_header.payload_size() {
                        <#field_type as IsoBox<'a>>::demux(
                            box_header,
                            ::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut payload_reader, size),
                        )
                    } else {
                        <#field_type as IsoBox<'a>>::demux(box_header, &mut payload_reader)
                    };

                    match res {
                        Ok(v) => v,
                        Err(e) if e.kind() == ::std::io::ErrorKind::UnexpectedEof => {
                            break;
                        },
                        Err(e) => return Err(e),
                    }
                };
            })
        } else {
            Ok(quote! {
                let #field_name = {
                    let box_header = #crate_path::BoxHeader::demux(&mut payload_reader)?;

                    if let Some(size) = box_header.payload_size() {
                        <#field_type as IsoBox<'a>>::demux(
                            box_header,
                            ::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut payload_reader, size),
                        )?
                    } else {
                        <#field_type as IsoBox<'a>>::demux(box_header, &mut payload_reader)?
                    }
                };
            })
        }
    } else if READ_FN_TYPES.contains(&field_type_str.as_str()) {
        let read_fn = format_ident!("read_{}", field_type_str);

        // u8 and i8 do not require endianness
        let generics = if field_type_str == "u8" || field_type_str == "i8" {
            None
        } else {
            Some(quote! {
                <::byteorder::BigEndian>
            })
        };

        if break_on_eof {
            Ok(quote! {
                let #field_name = match ::byteorder::ReadBytesExt::#read_fn::#generics(&mut ::scuffle_bytes_util::zero_copy::ZeroCopyReader::as_std(&mut payload_reader)) {
                    Ok(v) => v,
                    Err(e) if e.kind() == ::std::io::ErrorKind::UnexpectedEof => {
                        break;
                    },
                    Err(e) => return Err(e),
                };
            })
        } else {
            Ok(quote! {
                let #field_name = ::byteorder::ReadBytesExt::#read_fn::#generics(&mut ::scuffle_bytes_util::zero_copy::ZeroCopyReader::as_std(&mut payload_reader))?;
            })
        }
    } else if let syn::Type::Array(type_array) = field_type {
        let syn::Type::Path(syn::TypePath { path, .. }) = &*type_array.elem else {
            return Err(syn::Error::new_spanned(
                type_array,
                format!("Only u8 arrays supprted, found: {field_type_str}"),
            ));
        };

        if !path.is_ident("u8") {
            return Err(syn::Error::new_spanned(
                path,
                format!("Only u8 arrays supprted, found: {field_type_str}"),
            ));
        }

        let len = type_array.len.clone();

        if break_on_eof {
            Ok(quote! {
                let #field_name: #type_array = {
                    match ::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read(&mut payload_reader, #len) {
                        Ok(v) => ::std::result::Result::unwrap(::std::convert::TryInto::try_into(::scuffle_bytes_util::BytesCow::as_bytes(&v))),
                        Err(e) if e.kind() == ::std::io::ErrorKind::UnexpectedEof => {
                            break;
                        },
                        Err(e) => return Err(e),
                    }
                };
            })
        } else {
            Ok(quote! {
                let #field_name: #type_array = ::std::result::Result::unwrap(::std::convert::TryInto::try_into(::scuffle_bytes_util::BytesCow::as_bytes(&::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read(&mut payload_reader, #len)?)));
            })
        }
    } else {
        return Err(syn::Error::new_spanned(
            field_type,
            format!("Unsupported type: {field_type_str}"),
        ));
    }
}
