#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

use darling::FromDeriveInput;
use quote::{ToTokens, quote};
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
    #[darling(default = "default_crate_path")]
    crate_path: syn::Path,
    #[darling(default)]
    skip_deserialize_impl: bool,
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

    if let Some(field) = fields.iter().find(|f| f.header && (f.repeated || f.nested_box.is_some())) {
        return Err(syn::Error::new_spanned(
            field.ident.as_ref().expect("unreachable: only named fields supported"),
            "Cannot combine header and repeated or nested_box",
        ));
    }

    if let Some(field) = fields.iter().filter(|f| f.repeated).nth(1) {
        return Err(syn::Error::new_spanned(
            field.ident.as_ref().expect("unreachable: only named fields supported"),
            "Only one field can be marked as repeated",
        ));
    }

    if let Some(field) = fields.iter().filter(|f| f.header).nth(1) {
        return Err(syn::Error::new_spanned(
            field.ident.as_ref().expect("unreachable: only named fields supported"),
            "Only one field can be marked as header",
        ));
    }

    if fields.iter().any(|f| f.repeated) {
        if let Some(field) = fields.iter().find(|f| f.nested_box.is_some()) {
            return Err(syn::Error::new_spanned(
                field.ident.as_ref().expect("unreachable: only named fields supported"),
                "Cannot combine repeated and nested_box in the same struct",
            ));
        }
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
    repeated: bool,
    #[darling(default)]
    nested_box: Option<IsoBoxFieldNestedBox>,
}

#[derive(Debug, Default, darling::FromMeta, PartialEq, Eq, Clone, Copy)]
#[darling(default, from_word = default_field_collect)]
enum IsoBoxFieldNestedBox {
    #[default]
    Single,
    Collect,
    CollectUnknown,
}

fn default_field_collect() -> darling::Result<IsoBoxFieldNestedBox> {
    Ok(IsoBoxFieldNestedBox::default())
}

fn box_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let opts = IsoBoxOpts::from_derive_input(&input)?;
    let crate_path = opts.crate_path;

    let fields = into_fields_checked(opts.data)?;

    let header_constructor = fields.iter().find(|f| f.header).map(|f| {
        let name = f.ident.as_ref().expect("unreachable: only named fields supported");
        quote! {
            #name: header,
        }
    });

    let header_parser = quote! {
        {
            let header = <#crate_path::BoxHeader as ::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut reader)?;
            <<Self as #crate_path::IsoBox>::Header as ::scuffle_bytes_util::zero_copy::DeserializeSeed<#crate_path::BoxHeader>>::deserialize_seed(
                &mut reader,
                header,
            )?
        }
    };

    let mut fields_in_self = Vec::new();
    let mut field_parsers = Vec::new();

    for field in fields.iter().filter(|f| !f.header && f.nested_box.is_none()) {
        let field_name = field.ident.as_ref().expect("unreachable: only named fields supported");

        let read_field = if field.repeated {
            read_field_repeated(field, &crate_path)
        } else if field.from.is_some() {
            read_field_with_from(field)
        } else {
            read_field(field)
        };

        fields_in_self.push(quote! {
            #field_name,
        });
        field_parsers.push(quote! {
            let #field_name = #read_field;
        });
    }

    let collect_boxes = fields.iter().any(|f| f.nested_box.is_some());

    let box_parser = if collect_boxes {
        Some(nested_box_parser(fields.iter(), &crate_path))
    } else {
        None
    };

    for (field, nested) in fields.iter().filter_map(|f| f.nested_box.map(|n| (f, n))) {
        let field_type = &field.ty;
        let field_type = field_type.to_token_stream().to_string();
        let field_name = field.ident.clone().expect("unreachable: only named fields supported");

        match nested {
            IsoBoxFieldNestedBox::Single => {
                fields_in_self.push(quote! {
                    #field_name: ::std::option::Option::ok_or(#field_name, ::std::io::Error::new(::std::io::ErrorKind::InvalidData, format!("{} not found", #field_type)))?,
                });
            }
            IsoBoxFieldNestedBox::Collect | IsoBoxFieldNestedBox::CollectUnknown => {
                fields_in_self.push(quote! {
                    #field_name,
                });
            }
        }
    }

    let ident = opts.ident;
    let generics = opts.generics;
    let box_type = opts.box_type;
    let header_type = fields
        .iter()
        .find(|f| f.header)
        .map(|f| f.ty.to_token_stream())
        .ok_or(syn::Error::new_spanned(&ident, "No header field found"))?;

    let deserialize_impl = (!opts.skip_deserialize_impl).then_some(quote! {
        impl<'a> ::scuffle_bytes_util::zero_copy::Deserialize<'a> for #ident #generics {
            fn deserialize<R>(mut reader: R) -> ::std::io::Result<Self>
            where
                R: ::scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
            {
                let seed = #header_parser;
                <Self as ::scuffle_bytes_util::zero_copy::DeserializeSeed<#header_type>>::deserialize_seed(reader, seed)
            }
        }
    });

    let output = quote! {
        impl #generics IsoBox for #ident #generics {
            const TYPE: [u8; 4] = *#box_type;
            type Header = #header_type;
        }

        #deserialize_impl

        impl<'a> ::scuffle_bytes_util::zero_copy::DeserializeSeed<'a, #header_type> for #ident #generics {
            fn deserialize_seed<R>(mut reader: R, seed: #header_type) -> ::std::io::Result<Self>
            where
                R: ::scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
            {
                use ::scuffle_bytes_util::zero_copy::{I24Be as i24, I48Be as i48, U24Be as u24, U48Be as u48};
                let header = seed;

                #(#field_parsers)*
                #box_parser

                Ok(Self {
                    #(#fields_in_self)*
                    #header_constructor
                })
            }
        }
    };

    Ok(output)
}

fn nested_box_parser<'a>(fields: impl Iterator<Item = &'a IsoBoxField>, crate_path: &syn::Path) -> proc_macro2::TokenStream {
    let (inits, match_arms): (proc_macro2::TokenStream, proc_macro2::TokenStream) = fields
        .filter_map(|f| f.nested_box.as_ref().map(|n| (f, n)))
        .map(|(f, nested)| {
            let field_type = &f.ty;
            let field_name = f.ident.as_ref().expect("unreachable: only named fields supported");

            match nested {
                IsoBoxFieldNestedBox::Single => {
                    let init = quote! {
                        let mut #field_name = ::std::option::Option::None;
                    };
                    let match_arm = quote! {
                        #crate_path::BoxType::FourCc(<#field_type as #crate_path::IsoBox>::TYPE) => {
                            // If the type matches, we can deserialize the box header
                            // If the box header is a normal (partial) box header, this won't consume any bytes from the reader
                            let Some(box_header) = ::scuffle_bytes_util::IoResultExt::eof_to_none(
                                <<#field_type as #crate_path::IsoBox>::Header as ::scuffle_bytes_util::zero_copy::DeserializeSeed<#crate_path::BoxHeader>>::deserialize_seed(&mut reader, partial_box_header)
                            )? else {
                                // EOF
                                break;
                            };

                            if let Some(payload_size) = #crate_path::BoxHeaderProperties::payload_size(&box_header) {
                                // Initialize the payload reader with the payload size
                                let mut payload_reader = ::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut reader, #crate_path::BoxHeaderProperties::payload_size(&box_header).unwrap_or(1000));
                                // Deserialize the box payload
                                let Some(iso_box) = ::scuffle_bytes_util::IoResultExt::eof_to_none(
                                    <#field_type as ::scuffle_bytes_util::zero_copy::DeserializeSeed<<#field_type as #crate_path::IsoBox>::Header>>::deserialize_seed(
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
                                #field_name = ::std::option::Option::Some(iso_box);
                            } else {
                                // Deserialize the box payload
                                let Some(iso_box) = ::scuffle_bytes_util::IoResultExt::eof_to_none(
                                    <#field_type as ::scuffle_bytes_util::zero_copy::DeserializeSeed<<#field_type as #crate_path::IsoBox>::Header>>::deserialize_seed(
                                        &mut reader,
                                        box_header,
                                    )
                                )? else {
                                    // EOF
                                    break;
                                };
                                #field_name = ::std::option::Option::Some(iso_box);
                            }
                        }
                    };
                    (init, match_arm)
                }
                IsoBoxFieldNestedBox::Collect => {
                    let init = quote! {
                        let mut #field_name = <#field_type as ::std::default::Default>::default();
                    };
                    let match_arm = quote! {
                        #crate_path::BoxType::FourCc(<<#field_type as ::scuffle_bytes_util::zero_copy::Container>::Item as #crate_path::IsoBox>::TYPE) => {
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
                                ::scuffle_bytes_util::zero_copy::Container::add(&mut #field_name, iso_box);
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
                                ::scuffle_bytes_util::zero_copy::Container::add(&mut #field_name, iso_box);
                            }
                        }
                    };
                    (init, match_arm)
                }
                IsoBoxFieldNestedBox::CollectUnknown => {
                    let init = quote! {
                        let mut #field_name = <#field_type as ::std::default::Default>::default();
                    };
                    let match_arm = quote! {
                        _ => {
                            if let Some(payload_size) = #crate_path::BoxHeaderProperties::payload_size(&partial_box_header) {
                                let mut payload_reader = ::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut reader, payload_size);
                                let Some(unknown_box) = ::scuffle_bytes_util::IoResultExt::eof_to_none(
                                    <#crate_path::UnknownBox as ::scuffle_bytes_util::zero_copy::DeserializeSeed<'_, #crate_path::BoxHeader>>::deserialize_seed(&mut payload_reader, partial_box_header)
                                )? else {
                                    break;
                                };
                                ::scuffle_bytes_util::zero_copy::Container::add(&mut #field_name, unknown_box);
                                // Align the reader to the start of the next box
                                ::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read_to_end(&mut payload_reader)?;
                            } else {
                                let Some(unknown_box) = ::scuffle_bytes_util::IoResultExt::eof_to_none(
                                    <#crate_path::UnknownBox as ::scuffle_bytes_util::zero_copy::DeserializeSeed<'_, #crate_path::BoxHeader>>::deserialize_seed(&mut reader, partial_box_header)
                                )? else {
                                    break;
                                };
                                ::scuffle_bytes_util::zero_copy::Container::add(&mut #field_name, unknown_box);
                            }

                        }
                    };
                    (init, match_arm)
                }
            }
        })
        .collect();

    quote! {
        #inits
        loop {
            // Deserialize the box header which is part of every box
            let Some(partial_box_header) = ::scuffle_bytes_util::IoResultExt::eof_to_none(
                <#crate_path::BoxHeader as ::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut reader)
            )? else {
                // EOF
                break;
            };

            match #crate_path::BoxHeaderProperties::box_type(&partial_box_header) {
                #match_arms
                _ => {
                    // Ignore unknown boxes if we are not collecting them
                    // Align the reader to the start of the next box
                    if let Some(payload_size) = #crate_path::BoxHeaderProperties::payload_size(&partial_box_header) {
                        let mut payload_reader = ::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut reader, payload_size);
                        ::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read_to_end(&mut payload_reader)?;
                    } else {
                        ::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read_to_end(&mut reader)?;
                    }
                }
            }
        }
    }
}

fn read_field_repeated(field: &IsoBoxField, crate_path: &syn::Path) -> proc_macro2::TokenStream {
    let field_type = &field.ty;

    if let Some(from) = field.from.as_ref() {
        quote! {
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
        }
    } else {
        quote! {
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
        }
    }
}

fn read_field_with_from(field: &IsoBoxField) -> proc_macro2::TokenStream {
    let field_type = &field.ty;
    let read_field = read_field(field);

    quote! {
        <#field_type as ::std::convert::From<_>>::from(#read_field)
    }
}

fn read_field(field: &IsoBoxField) -> proc_macro2::TokenStream {
    let field_type = field.from.as_ref().unwrap_or(&field.ty);

    quote! {
        <#field_type as ::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut reader)?
    }
}
