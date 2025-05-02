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
    generics: syn::Generics,
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

fn into_fields_checked(data: darling::ast::Data<(), IsoBoxField>) -> syn::Result<darling::ast::Fields<IsoBoxField>> {
    let fields = data.take_struct().expect("unreachable: only structs supported");

    if let Some(field) = fields.iter().find(|f| f.header && f.from.is_some()) {
        return Err(syn::Error::new_spanned(
            field.ident.as_ref().expect("unreachable: only named fields supported"),
            "Cannot combine header and from",
        ));
    }

    if let Some(field) = fields.iter().find(|f| f.repeated.is_some() && f.header) {
        return Err(syn::Error::new_spanned(
            field.ident.as_ref().expect("unreachable: only named fields supported"),
            "Cannot combine repeated and header",
        ));
    }

    if let Some(field) = fields
        .iter()
        .filter(|f| {
            f.repeated
                .is_some_and(|r| r == IsoBoxFieldRepeated::Field || r == IsoBoxFieldRepeated::Box)
        })
        .nth(1)
    {
        return Err(syn::Error::new_spanned(
            field.ident.as_ref().expect("unreachable: only named fields supported"),
            "Only one field can be marked as repeated",
        ));
    }

    if let Some(field) = fields
        .iter()
        .find(|f| f.repeated.is_some_and(|r| r == IsoBoxFieldRepeated::UnknownBox))
    {
        if !fields
            .iter()
            .any(|f| f.repeated.is_some_and(|r| r == IsoBoxFieldRepeated::Box))
        {
            return Err(syn::Error::new_spanned(
                field.ident.as_ref().expect("unreachable: only named fields supported"),
                "Cannot collect unknown boxes without a repeated boxes field",
            ));
        }
    }

    if let Some(field) = fields.iter().filter(|f| f.header).nth(1) {
        return Err(syn::Error::new_spanned(
            field.ident.as_ref().expect("unreachable: only named fields supported"),
            "Only one field can be marked as header",
        ));
    }

    Ok(fields)
}

#[derive(Debug, darling::FromField, Clone)]
#[darling(attributes(iso_box))]
struct IsoBoxField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    header: bool,
    #[darling(default)]
    from: Option<syn::Type>,
    #[darling(default)]
    repeated: Option<IsoBoxFieldRepeated>,
}

#[derive(Debug, Default, darling::FromMeta, PartialEq, Eq, Clone, Copy)]
#[darling(default, from_word = iso_box_field_repeated_from_word)]
enum IsoBoxFieldRepeated {
    #[default]
    Field,
    Box,
    UnknownBox,
}

fn iso_box_field_repeated_from_word() -> darling::Result<IsoBoxFieldRepeated> {
    Ok(IsoBoxFieldRepeated::default())
}

fn box_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let opts = IsoBoxOpts::from_derive_input(&input)?;
    let crate_path = opts.crate_path;

    let mut constructor_fields = Vec::new();

    let fields = into_fields_checked(opts.data)?;

    let header_constructor = fields.iter().find(|f| f.header).map(|f| {
        let name = f.ident.as_ref().expect("unreachable: only named fields supported");
        quote! {
            #name: header,
        }
    });

    let header_parser = quote! {
        <<Self as #crate_path::IsoBox>::Header as ::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut reader)?
    };

    let unknown_boxes_field = fields
        .iter()
        .find(|f| f.repeated.as_ref().is_some_and(|r| *r == IsoBoxFieldRepeated::UnknownBox))
        .cloned();

    let unknown_boxes = if let Some(field) = &unknown_boxes_field {
        let field_type = &field.ty;
        Some(quote! {
            let mut unknown_boxes = <#field_type as ::std::default::Default>::default();
        })
    } else {
        None
    };

    for field in fields.into_iter().filter(|f| !f.header) {
        let field_name = field.ident.clone().expect("unreachable: only named fields supported");

        let read_field = if field.repeated.is_some() {
            read_field_repeated(field, &crate_path, unknown_boxes_field.as_ref())
        } else if field.from.is_some() {
            Some(read_field_with_from(field))
        } else {
            Some(read_field(field))
        };

        if let Some(read_field) = read_field {
            constructor_fields.push(quote! {
                #field_name: #read_field,
            });
        } else {
            constructor_fields.push(quote! {
                #field_name,
            });
        }
    }

    let box_type = opts.box_type;
    let ident = opts.ident;
    let generics = opts.generics;
    let header_type = if opts.full_header {
        quote! { #crate_path::FullBoxHeader }
    } else {
        quote! { #crate_path::BoxHeader }
    };

    let output = quote! {
        impl #generics IsoBox for #ident #generics {
            const TYPE: [u8; 4] = *#box_type;
            type Header = #header_type;
        }

        impl<'a> ::scuffle_bytes_util::zero_copy::DeserializeSeed<'a, #header_type> for #ident #generics {
            fn deserialize_seed<R>(mut reader: R, seed: #header_type) -> ::std::io::Result<Self>
            where
                R: ::scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
            {
                use ::scuffle_bytes_util::zero_copy::{I24Be as i24, I48Be as i48, U24Be as u24, U48Be as u48};
                let header = seed;
                #unknown_boxes
                Ok(Self {
                    #(#constructor_fields)*
                    #header_constructor
                })
            }
        }

        impl<'a> ::scuffle_bytes_util::zero_copy::Deserialize<'a> for #ident #generics {
            fn deserialize<R>(mut reader: R) -> ::std::io::Result<Self>
            where
                R: ::scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
            {
                let seed = #header_parser;
                <Self as ::scuffle_bytes_util::zero_copy::DeserializeSeed<#header_type>>::deserialize_seed(reader, seed)
            }
        }
    };

    Ok(output)
}

fn read_field_repeated(
    field: IsoBoxField,
    crate_path: &syn::Path,
    unknown_boxes_field: Option<&IsoBoxField>,
) -> Option<proc_macro2::TokenStream> {
    let field_type = field.ty;
    let repeated = field.repeated.unwrap();

    match repeated {
        IsoBoxFieldRepeated::Field => {
            if let Some(from) = field.from {
                Some(quote! {
                    {
                        if let Some(payload_size) = #crate_path::BoxHeaderProperties::payload_size(&header) {
                            let mut payload_reader = ::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut reader, payload_size);
                            let iter = ::std::iter::from_fn(||
                                ::std::result::Result::transpose(::scuffle_bytes_util::IoResultExt::eof_to_none(
                                    <#from as ::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut payload_reader)
                                ))
                            );
                            let iter = ::std::iter::Iterator::map(iter, |item| {
                                match item {
                                    Ok(item) => Ok(::std::convert::From::from(item)),
                                    Err(e) => Err(e),
                                }
                            });
                            ::std::iter::Iterator::collect::<::std::result::Result<#field_type, ::std::io::Error>>(iter)?
                        } else {
                            let iter = ::std::iter::from_fn(||
                                ::std::result::Result::transpose(::scuffle_bytes_util::IoResultExt::eof_to_none(
                                    <#from as ::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut reader)
                                ))
                            );
                            let iter = ::std::iter::Iterator::map(iter, |item| {
                                match item {
                                    Ok(item) => Ok(::std::convert::From::from(item)),
                                    Err(e) => Err(e),
                                }
                            });
                            ::std::iter::Iterator::collect::<::std::result::Result<#field_type, ::std::io::Error>>(iter)?
                        }
                    }
                })
            } else {
                Some(quote! {
                    {
                        if let Some(payload_size) = #crate_path::BoxHeaderProperties::payload_size(&header) {
                            let mut payload_reader = ::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut reader, payload_size);
                            let iter = ::std::iter::from_fn(||
                                ::std::result::Result::transpose(::scuffle_bytes_util::IoResultExt::eof_to_none(
                                    <<#field_type as ::scuffle_bytes_util::zero_copy::Container>::Item as ::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut payload_reader)
                                ))
                            );
                            ::std::iter::Iterator::collect::<::std::result::Result<#field_type, ::std::io::Error>>(iter)?
                        } else {
                            let iter = ::std::iter::from_fn(||
                                ::std::result::Result::transpose(::scuffle_bytes_util::IoResultExt::eof_to_none(
                                    <<#field_type as ::scuffle_bytes_util::zero_copy::Container>::Item as ::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut reader)
                                ))
                            );
                            ::std::iter::Iterator::collect::<::std::result::Result<#field_type, ::std::io::Error>>(iter)?
                        }
                    }
                })
            }
        }
        IsoBoxFieldRepeated::Box => {
            let unknown_box = if let Some(field) = unknown_boxes_field {
                let field_name = field.ident.as_ref().expect("unreachable: only named fields supported");
                Some(quote! {
                    let mut payload_reader = ::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut reader, #crate_path::BoxHeaderProperties::payload_size(&partial_box_header).unwrap_or(1000));
                    let Some(unknown_box) = ::scuffle_bytes_util::IoResultExt::eof_to_none(
                        <#crate_path::UnknownBox as ::scuffle_bytes_util::zero_copy::DeserializeSeed<'_, #crate_path::BoxHeader>>::deserialize_seed(&mut payload_reader, partial_box_header)
                    )? else {
                        break;
                    };
                    ::scuffle_bytes_util::zero_copy::Container::add(&mut #field_name, unknown_box);
                    // Align the reader to the start of the next box
                    ::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read_to_end(&mut payload_reader)?;
                })
            } else {
                None
            };

            Some(quote! {
                {
                    let mut container = <#field_type as ::std::default::Default>::default();
                    loop {
                        // Deserialize the box header which is part of every box
                        let Some(partial_box_header) = ::scuffle_bytes_util::IoResultExt::eof_to_none(
                            <#crate_path::BoxHeader as ::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut reader)
                        )? else {
                            // EOF
                            break;
                        };

                        // If the box type doesn't match the expected type, we have an unknown box
                        if !#crate_path::BoxType::is_four_cc(&#crate_path::BoxHeaderProperties::box_type(&partial_box_header), &<<#field_type as ::scuffle_bytes_util::zero_copy::Container>::Item as #crate_path::IsoBox>::TYPE) {
                            #unknown_box
                            continue;
                        }

                        // If the type matches, we can deserialize the box header
                        // If the box header is a normal (partial) box header, this won't consume any bytes from the reader
                        let Some(box_header) = ::scuffle_bytes_util::IoResultExt::eof_to_none(
                            <<<#field_type as ::scuffle_bytes_util::zero_copy::Container>::Item as #crate_path::IsoBox>::Header as ::scuffle_bytes_util::zero_copy::DeserializeSeed<#crate_path::BoxHeader>>::deserialize_seed(&mut reader, partial_box_header)
                        )? else {
                            // EOF
                            break;
                        };

                        if let Some(payload_size) = #crate_path::BoxHeaderProperties::payload_size(&box_header) {
                            // Initialize the payload reader with the payload size
                            let mut payload_reader = ::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut reader, #crate_path::BoxHeaderProperties::payload_size(&box_header).unwrap_or(1000));
                            // Deserialize the box payload
                            let Some(iso_box) = ::scuffle_bytes_util::IoResultExt::eof_to_none(
                                <<#field_type as ::scuffle_bytes_util::zero_copy::Container>::Item as ::scuffle_bytes_util::zero_copy::DeserializeSeed<<<#field_type as ::scuffle_bytes_util::zero_copy::Container>::Item as #crate_path::IsoBox>::Header>>::deserialize_seed(
                                    &mut payload_reader,
                                    box_header,
                                )
                            )? else {
                                // EOF
                                // Align the reader to the start of the next box
                                ::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read_to_end(&mut payload_reader)?;
                                break;
                            };
                            // Align the reader to the start of the next box
                            ::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read_to_end(&mut payload_reader)?;
                            ::scuffle_bytes_util::zero_copy::Container::add(&mut container, iso_box);
                        } else {
                            // Deserialize the box payload
                            let Some(iso_box) = ::scuffle_bytes_util::IoResultExt::eof_to_none(
                                <<#field_type as ::scuffle_bytes_util::zero_copy::Container>::Item as ::scuffle_bytes_util::zero_copy::DeserializeSeed<<<#field_type as ::scuffle_bytes_util::zero_copy::Container>::Item as #crate_path::IsoBox>::Header>>::deserialize_seed(
                                    &mut reader,
                                    box_header,
                                )
                            )? else {
                                // EOF
                                break;
                            };
                            ::scuffle_bytes_util::zero_copy::Container::add(&mut container, iso_box);
                        }
                    }
                    container
                }
            })
        }
        _ => None,
    }
}

fn read_field_with_from(field: IsoBoxField) -> proc_macro2::TokenStream {
    let field_type = field.ty.clone();
    let read_field = read_field(field);

    quote! {
        <#field_type as ::std::convert::From<_>>::from(#read_field)
    }
}

fn read_field(field: IsoBoxField) -> proc_macro2::TokenStream {
    let field_type = field.from.unwrap_or(field.ty);

    quote! {
        <#field_type as ::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut reader)?
    }
}
