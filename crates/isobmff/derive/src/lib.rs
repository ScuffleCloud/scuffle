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
    box_type: Option<syn::LitByteStr>,
    #[darling(default = "default_crate_path")]
    crate_path: syn::Path,
    #[darling(default)]
    skip_impl: Option<SkipImpls>,
}

fn default_crate_path() -> syn::Path {
    syn::parse_str("::isobmff").unwrap()
}

#[derive(Debug)]
struct SkipImpls(Vec<SkipImpl>);

impl darling::FromMeta for SkipImpls {
    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let skips = items
            .iter()
            .map(|m| match m {
                darling::ast::NestedMeta::Meta(mi) => {
                    if let Some(ident) = mi.path().get_ident() {
                        SkipImpl::from_string(ident.to_string().as_str())
                    } else {
                        Ok(SkipImpl::All)
                    }
                }
                darling::ast::NestedMeta::Lit(lit) => SkipImpl::from_value(lit),
            })
            .collect::<Result<_, _>>()?;
        Ok(SkipImpls(skips))
    }

    fn from_word() -> darling::Result<Self> {
        Ok(SkipImpls(vec![SkipImpl::All]))
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(SkipImpls(vec![SkipImpl::from_string(value)?]))
    }
}

impl SkipImpls {
    fn should_impl(&self, this_impl: SkipImpl) -> bool {
        if self.0.contains(&SkipImpl::All) {
            // Nothing should be implemented when all impls are skipped
            return false;
        }

        if self.0.contains(&this_impl) {
            return false;
        }

        true
    }
}

#[derive(Debug, PartialEq, Eq, darling::FromMeta)]
enum SkipImpl {
    All,
    Deserialize,
    DeserializeSeed,
    Serialize,
    Sized,
    IsoBox,
}

fn into_fields_checked(data: darling::ast::Data<(), IsoBoxField>) -> syn::Result<darling::ast::Fields<IsoBoxField>> {
    let fields = data.take_struct().expect("unreachable: only structs supported");

    if let Some(field) = fields.iter().filter(|f| f.repeated).nth(1) {
        return Err(syn::Error::new_spanned(
            field.ident.as_ref().expect("unreachable: only named fields supported"),
            "Only one field can be marked as repeated",
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

    let mut fields_in_self = Vec::new();
    let mut field_parsers = Vec::new();
    let mut field_serializers = Vec::new();

    for field in fields.iter().filter(|f| f.nested_box.is_none()) {
        let field_name = field.ident.as_ref().expect("unreachable: only named fields supported");

        let read_field = if field.repeated {
            read_field_repeated(field, &crate_path)
        } else if field.from.is_some() {
            read_field_with_from(field, &crate_path)
        } else {
            read_field(field, &crate_path)
        };

        fields_in_self.push(field_name.to_token_stream());
        field_parsers.push(quote! {
            let #field_name = #read_field;
        });

        match (field.repeated, &field.from) {
            (true, None) => {
                field_serializers.push(quote! {
                    for item in &self.#field_name {
                        #crate_path::reexports::scuffle_bytes_util::zero_copy::Serialize::serialize(item, &mut writer)?;
                    }
                });
            }
            (true, Some(from_ty)) => {
                field_serializers.push(quote! {
                    for item in &self.#field_name {
                        #crate_path::reexports::scuffle_bytes_util::zero_copy::Serialize::serialize(&::std::convert::Into::<#from_ty>::into(*item), &mut writer)?;
                    }
                });
            }
            (false, None) => {
                field_serializers.push(quote! {
                    #crate_path::reexports::scuffle_bytes_util::zero_copy::Serialize::serialize(&self.#field_name, &mut writer)?;
                });
            }
            (false, Some(from_ty)) => {
                field_serializers.push(quote! {
                    #crate_path::reexports::scuffle_bytes_util::zero_copy::Serialize::serialize(&::std::convert::Into::<#from_ty>::into(self.#field_name), &mut writer)?;
                });
            }
        }
    }

    let collect_boxes = fields.iter().any(|f| f.nested_box.is_some());

    let box_parser = if collect_boxes {
        Some(nested_box_parser(fields.iter(), &crate_path))
    } else {
        None
    };

    for (field, nested) in fields.iter().filter_map(|f| f.nested_box.map(|n| (f, n))) {
        let field_name = field.ident.clone().expect("unreachable: only named fields supported");
        let field_name_str = field_name.to_string();

        match nested {
            IsoBoxFieldNestedBox::Single => {
                fields_in_self.push(quote! {
                    #field_name: ::std::option::Option::ok_or(#field_name, ::std::io::Error::new(::std::io::ErrorKind::InvalidData, format!("{} not found", #field_name_str)))?
                });
                field_serializers.push(quote! {
                    #crate_path::reexports::scuffle_bytes_util::zero_copy::Serialize::serialize(&self.#field_name, &mut writer)?;
                });
            }
            IsoBoxFieldNestedBox::Collect | IsoBoxFieldNestedBox::CollectUnknown => {
                fields_in_self.push(field_name.to_token_stream());
                field_serializers.push(quote! {
                    #[allow(for_loops_over_fallibles)]
                    for item in &self.#field_name {
                        #crate_path::reexports::scuffle_bytes_util::zero_copy::Serialize::serialize(item, &mut writer)?;
                    }
                });
            }
        }
    }

    let ident = opts.ident;
    let generics = opts.generics;

    let mut impls = Vec::new();

    if opts.skip_impl.as_ref().is_none_or(|s| s.should_impl(SkipImpl::IsoBox)) {
        let box_type = opts.box_type.ok_or(syn::Error::new_spanned(
            &ident,
            "box_type is required for IsoBox (use skip_impl(iso_box) to skip this impl)",
        ))?;

        impls.push(quote! {
            #[automatically_derived]
            impl #generics IsoBox for #ident #generics {
                const TYPE: #crate_path::BoxType = #crate_path::BoxType::FourCc(*#box_type);
            }
        });
    }

    if opts.skip_impl.as_ref().is_none_or(|s| s.should_impl(SkipImpl::Deserialize)) {
        impls.push(quote! {
            #[automatically_derived]
            impl<'a> #crate_path::reexports::scuffle_bytes_util::zero_copy::Deserialize<'a> for #ident #generics {
                fn deserialize<R>(mut reader: R) -> ::std::io::Result<Self>
                where
                    R: #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
                {
                    let seed = <#crate_path::BoxHeader as #crate_path::reexports::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut reader)?;
                    <Self as #crate_path::reexports::scuffle_bytes_util::zero_copy::DeserializeSeed<#crate_path::BoxHeader>>::deserialize_seed(reader, seed)
                }
            }
        });
    }

    if opts
        .skip_impl
        .as_ref()
        .is_none_or(|s| s.should_impl(SkipImpl::DeserializeSeed))
    {
        impls.push(quote! {
            #[automatically_derived]
            impl<'a> #crate_path::reexports::scuffle_bytes_util::zero_copy::DeserializeSeed<'a, #crate_path::BoxHeader> for #ident #generics {
                fn deserialize_seed<R>(mut reader: R, seed: #crate_path::BoxHeader) -> ::std::io::Result<Self>
                where
                    R: #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
                {
                    #(#field_parsers)*
                    #box_parser

                    Ok(Self {
                        #(#fields_in_self,)*
                    })
                }
            }
        });
    }

    if opts.skip_impl.as_ref().is_none_or(|s| s.should_impl(SkipImpl::Serialize)) {
        impls.push(quote! {
            #[automatically_derived]
            impl #generics #crate_path::reexports::scuffle_bytes_util::zero_copy::Serialize for #ident #generics {
                fn serialize<W>(&self, mut writer: W) -> ::std::io::Result<()>
                where
                    W: ::std::io::Write
                {
                    <Self as #crate_path::IsoBox>::serialize_box_header(self, &mut writer)?;
                    #(#field_serializers)*
                    Ok(())
                }
            }
        });
    }

    if opts.skip_impl.as_ref().is_none_or(|s| s.should_impl(SkipImpl::Sized)) {
        let field_names = fields
            .fields
            .iter()
            .map(|f| f.ident.clone().expect("unreachable: only named fields supported"))
            .collect::<Vec<_>>();

        impls.push(quote! {
            #[automatically_derived]
            impl #generics #crate_path::IsoSized for #ident #generics {
                fn size(&self) -> usize {
                    <Self as #crate_path::IsoBox>::add_header_size(#(#crate_path::IsoSized::size(&self.#field_names))+*)
                }
            }
        });
    }

    Ok(impls.into_iter().collect())
}

fn nested_box_parser<'a>(fields: impl Iterator<Item = &'a IsoBoxField>, crate_path: &syn::Path) -> proc_macro2::TokenStream {
    let mut inits = Vec::new();
    let mut match_arms = Vec::new();
    let mut catch_all_arms = Vec::new();

    for (f, nested) in fields.filter_map(|f| f.nested_box.as_ref().map(|n| (f, n))) {
        let field_type = &f.ty;
        let field_name = f.ident.as_ref().expect("unreachable: only named fields supported");

        match nested {
            IsoBoxFieldNestedBox::Single => {
                inits.push(quote! {
                    let mut #field_name = ::std::option::Option::None;
                });
                match_arms.push(quote! {
                    <#field_type as #crate_path::IsoBox>::TYPE => {
                        if let Some(payload_size) = #crate_path::BoxHeader::payload_size(&box_header) {
                            // Initialize the payload reader with the payload size
                            let mut payload_reader = #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut reader, payload_size);
                            // Deserialize the box payload
                            let Some(iso_box) = #crate_path::reexports::scuffle_bytes_util::IoResultExt::eof_to_none(
                                <#field_type as #crate_path::reexports::scuffle_bytes_util::zero_copy::DeserializeSeed<#crate_path::BoxHeader>>::deserialize_seed(
                                    &mut payload_reader,
                                    box_header,
                                )
                            )? else {
                                // EOF
                                // Align the reader to the start of the next box
                                #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read_to_end(&mut payload_reader)?;
                                break;
                            };
                            // Align the reader to the start of the next box
                            #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read_to_end(&mut payload_reader)?;
                            #field_name = ::std::option::Option::Some(iso_box);
                        } else {
                            // Deserialize the box payload
                            let Some(iso_box) = #crate_path::reexports::scuffle_bytes_util::IoResultExt::eof_to_none(
                                <#field_type as #crate_path::reexports::scuffle_bytes_util::zero_copy::DeserializeSeed<#crate_path::BoxHeader>>::deserialize_seed(
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
                });
            }
            IsoBoxFieldNestedBox::Collect => {
                inits.push(quote! {
                    let mut #field_name = <#field_type as ::std::default::Default>::default();
                });
                match_arms.push(quote! {
                    <<#field_type as #crate_path::reexports::scuffle_bytes_util::zero_copy::Container>::Item as #crate_path::IsoBox>::TYPE => {
                        if let Some(payload_size) = #crate_path::BoxHeader::payload_size(&box_header) {
                            // Initialize the payload reader with the payload size
                            let mut payload_reader = #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut reader, payload_size);
                            // Deserialize the box payload
                            let Some(iso_box) = #crate_path::reexports::scuffle_bytes_util::IoResultExt::eof_to_none(
                                <<#field_type as #crate_path::reexports::scuffle_bytes_util::zero_copy::Container>::Item as #crate_path::reexports::scuffle_bytes_util::zero_copy::DeserializeSeed<#crate_path::BoxHeader>>::deserialize_seed(
                                    &mut payload_reader,
                                    box_header,
                                )
                            )? else {
                                // EOF
                                // Align the reader to the start of the next box
                                #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read_to_end(&mut payload_reader)?;
                                break;
                            };
                            // Align the reader to the start of the next box
                            #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read_to_end(&mut payload_reader)?;
                            #crate_path::reexports::scuffle_bytes_util::zero_copy::Container::add(&mut #field_name, iso_box);
                        } else {
                            // Deserialize the box payload
                            let Some(iso_box) = #crate_path::reexports::scuffle_bytes_util::IoResultExt::eof_to_none(
                                <<#field_type as #crate_path::reexports::scuffle_bytes_util::zero_copy::Container>::Item as #crate_path::reexports::scuffle_bytes_util::zero_copy::DeserializeSeed<#crate_path::BoxHeader>>::deserialize_seed(
                                    &mut reader,
                                    box_header,
                                )
                            )? else {
                                // EOF
                                break;
                            };
                            #crate_path::reexports::scuffle_bytes_util::zero_copy::Container::add(&mut #field_name, iso_box);
                        }
                    }
                });
            }
            IsoBoxFieldNestedBox::CollectUnknown => {
                inits.push(quote! {
                    let mut #field_name = <#field_type as ::std::default::Default>::default();
                });
                catch_all_arms.push(quote! {
                    _ => {
                        if let Some(payload_size) = #crate_path::BoxHeader::payload_size(&box_header) {
                            let mut payload_reader = #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut reader, payload_size);
                            let Some(unknown_box) = #crate_path::reexports::scuffle_bytes_util::IoResultExt::eof_to_none(
                                <#crate_path::UnknownBox as #crate_path::reexports::scuffle_bytes_util::zero_copy::DeserializeSeed<'_, #crate_path::BoxHeader>>::deserialize_seed(&mut payload_reader, box_header)
                            )? else {
                                break;
                            };
                            #crate_path::reexports::scuffle_bytes_util::zero_copy::Container::add(&mut #field_name, unknown_box);
                            // Align the reader to the start of the next box
                            #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read_to_end(&mut payload_reader)?;
                        } else {
                            let Some(unknown_box) = #crate_path::reexports::scuffle_bytes_util::IoResultExt::eof_to_none(
                                <#crate_path::UnknownBox as #crate_path::reexports::scuffle_bytes_util::zero_copy::DeserializeSeed<'_, #crate_path::BoxHeader>>::deserialize_seed(&mut reader, box_header)
                            )? else {
                                break;
                            };
                            #crate_path::reexports::scuffle_bytes_util::zero_copy::Container::add(&mut #field_name, unknown_box);
                        }

                    }
                });
            }
        }
    }

    quote! {
        #(#inits)*
        loop {
            // Deserialize the box header which is part of every box
            let Some(box_header) = #crate_path::reexports::scuffle_bytes_util::IoResultExt::eof_to_none(
                <#crate_path::BoxHeader as #crate_path::reexports::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut reader)
            )? else {
                // EOF
                break;
            };

            match box_header.box_type {
                #(#match_arms)*
                #(#catch_all_arms)*
                _ => {
                    // Ignore unknown boxes if we are not collecting them
                    // Align the reader to the start of the next box
                    if let Some(payload_size) = #crate_path::BoxHeader::payload_size(&box_header) {
                        let mut payload_reader = #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut reader, payload_size);
                        #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read_to_end(&mut payload_reader)?;
                    } else {
                        #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader::try_read_to_end(&mut reader)?;
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
                if let Some(payload_size) = #crate_path::BoxHeader::payload_size(&seed) {
                    let mut payload_reader = #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut reader, payload_size);
                    let iter = ::std::iter::from_fn(||
                        ::std::result::Result::transpose(#crate_path::reexports::scuffle_bytes_util::IoResultExt::eof_to_none(
                            <#from as #crate_path::reexports::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut payload_reader)
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
                        ::std::result::Result::transpose(#crate_path::reexports::scuffle_bytes_util::IoResultExt::eof_to_none(
                            <#from as #crate_path::reexports::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut reader)
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
                if let Some(payload_size) = #crate_path::BoxHeader::payload_size(&seed) {
                    let mut payload_reader = #crate_path::reexports::scuffle_bytes_util::zero_copy::ZeroCopyReader::take(&mut reader, payload_size);
                    let iter = ::std::iter::from_fn(||
                        ::std::result::Result::transpose(#crate_path::reexports::scuffle_bytes_util::IoResultExt::eof_to_none(
                            <<#field_type as #crate_path::reexports::scuffle_bytes_util::zero_copy::Container>::Item as #crate_path::reexports::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut payload_reader)
                        ))
                    );
                    ::std::iter::Iterator::collect::<::std::result::Result<#field_type, ::std::io::Error>>(iter)?
                } else {
                    let iter = ::std::iter::from_fn(||
                        ::std::result::Result::transpose(#crate_path::reexports::scuffle_bytes_util::IoResultExt::eof_to_none(
                            <<#field_type as #crate_path::reexports::scuffle_bytes_util::zero_copy::Container>::Item as #crate_path::reexports::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut reader)
                        ))
                    );
                    ::std::iter::Iterator::collect::<::std::result::Result<#field_type, ::std::io::Error>>(iter)?
                }
            }
        }
    }
}

fn read_field_with_from(field: &IsoBoxField, crate_path: &syn::Path) -> proc_macro2::TokenStream {
    let field_type = &field.ty;
    let read_field = read_field(field, crate_path);

    quote! {
        <#field_type as ::std::convert::From<_>>::from(#read_field)
    }
}

fn read_field(field: &IsoBoxField, crate_path: &syn::Path) -> proc_macro2::TokenStream {
    let field_type = field.from.as_ref().unwrap_or(&field.ty);

    quote! {
        <#field_type as #crate_path::reexports::scuffle_bytes_util::zero_copy::Deserialize>::deserialize(&mut reader)?
    }
}
