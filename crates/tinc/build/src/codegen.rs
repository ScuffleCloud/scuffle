use std::collections::BTreeMap;

use anyhow::Context;
use convert_case::{Boundary, Case, Casing};
use quote::quote;
use syn::{Ident, parse_quote};
use tinc_pb::schema_oneof_options::Tagged;

use crate::extensions::{
    EnumOpts, Extensions, FieldKind, FieldVisibility, MessageOpts, MethodIo, PrimitiveKind, ServiceOpts, WellKnownType,
};

fn message_attributes(key: &str, prost: &mut tonic_build::Config) {
    prost.message_attribute(key, "#[derive(::tinc::reexports::serde::Serialize)]");
    prost.message_attribute(key, "#[derive(::tinc::reexports::serde::Deserialize)]");
    prost.message_attribute(key, "#[derive(::tinc::reexports::schemars::JsonSchema)]");
    prost.message_attribute(key, "#[serde(crate = \"::tinc::reexports::serde\")]");
    prost.message_attribute(key, "#[schemars(crate = \"::tinc::reexports::schemars\")]");
    prost.message_attribute(key, "#[schemars(deny_unknown_fields)]");
    prost.message_attribute(key, format!("#[schemars(rename = \"{key}\")]"));
}

fn enum_attributes(key: &str, prost: &mut tonic_build::Config, repr_enum: bool) {
    if repr_enum {
        prost.enum_attribute(key, "#[derive(::tinc::reexports::serde_repr::Serialize_repr)]");
        prost.enum_attribute(key, "#[derive(::tinc::reexports::serde_repr::Deserialize_repr)]");
        prost.enum_attribute(key, "#[derive(::tinc::reexports::schemars::JsonSchema_repr)]");
    } else {
        prost.enum_attribute(key, "#[derive(::tinc::reexports::serde::Serialize)]");
        prost.enum_attribute(key, "#[derive(::tinc::reexports::serde::Deserialize)]");
        prost.enum_attribute(key, "#[derive(::tinc::reexports::schemars::JsonSchema)]");
    }
    prost.enum_attribute(key, "#[serde(crate = \"::tinc::reexports::serde\")]");
    prost.enum_attribute(key, "#[schemars(crate = \"::tinc::reexports::schemars\")]");
    prost.enum_attribute(key, format!("#[schemars(rename = \"{key}\")]"));
}

fn field_omitable(key: &str, prost: &mut tonic_build::Config) {
    prost.field_attribute(key, "#[serde(default)]");
}

fn field_visibility(key: &str, prost: &mut tonic_build::Config, visibility: Option<FieldVisibility>) {
    match visibility {
        Some(FieldVisibility::Skip) => prost.field_attribute(key, "#[serde(skip)]"),
        Some(FieldVisibility::InputOnly) => prost.field_attribute(key, "#[serde(skip_serializing)]"),
        Some(FieldVisibility::OutputOnly) => prost.field_attribute(key, "#[serde(skip_deserializing)]"),
        _ => return,
    };
}

fn rename_all(key: &str, style: Option<i32>, prost: &mut tonic_build::Config, is_enum: bool) -> bool {
    let Some(style) = style
        .and_then(|style| tinc_pb::RenameAll::try_from(style).ok())
        .and_then(rename_all_to_serde_rename_all)
    else {
        return false;
    };

    let attr = format!("#[serde(rename_all = \"{style}\")]");
    if is_enum {
        prost.enum_attribute(key, &attr);
    } else {
        prost.message_attribute(key, &attr);
    }

    true
}

fn serde_rename(key: &str, name: &str, prost: &mut tonic_build::Config) {
    prost.field_attribute(key, format!("#[serde(rename = \"{name}\")]"));
}

fn get_common_import(start: &str, end: &str) -> String {
    let start_parts: Vec<&str> = start.split('.').collect();
    let end_parts: Vec<&str> = end.split('.').collect();

    // Find common prefix length
    let common_len = start_parts.iter().zip(&end_parts).take_while(|(a, b)| a == b).count();

    // Number of `super::` needed
    let num_supers = start_parts.len() - common_len - 2;
    let super_prefix = "super::".repeat(num_supers);

    // Remaining path from the common ancestor
    let relative_path = end_parts[common_len..].join("::");

    // Construct the final result
    format!("{}{}", super_prefix, relative_path)
}

fn with_attr(key: &str, mut field_kind: &FieldKind, nullable: bool, omitable: bool, prost: &mut tonic_build::Config) {
    fn schemars_with(field_kind: &FieldKind, current_namespace: &str) -> Option<String> {
        match field_kind {
            FieldKind::WellKnown(well_known) => Some(format!("::tinc::helpers::well_known::{}", well_known.name())),
            FieldKind::Optional(inner) => Some(format!(
                "::core::option::Option<{}>",
                schemars_with(inner, current_namespace)?
            )),
            FieldKind::List(inner) => Some(format!("::std::vec::Vec<{}>", schemars_with(inner, current_namespace)?)),
            FieldKind::Map(key, inner) => Some(format!(
                "::std::collections::HashMap<{}, {}>",
                match key {
                    PrimitiveKind::String => "::std::string::String",
                    PrimitiveKind::Bool => "::core::primitive::bool",
                    PrimitiveKind::I32 => "::core::primitive::i32",
                    PrimitiveKind::I64 => "::core::primitive::i64",
                    PrimitiveKind::U32 => "::core::primitive::u32",
                    PrimitiveKind::U64 => "::core::primitive::u64",
                    PrimitiveKind::Bytes => unimplemented!("map keys cannot be bytes"),
                    PrimitiveKind::F32 => unimplemented!("map keys cannot be f32"),
                    PrimitiveKind::F64 => unimplemented!("map keys cannot be f64"),
                },
                schemars_with(inner, current_namespace)?,
            )),
            FieldKind::Enum(name) => Some(get_common_import(current_namespace, name)),
            FieldKind::Primitive(_) => None,
            FieldKind::Message(_) => None,
        }
    }

    let is_optional = matches!(field_kind, FieldKind::Optional(_));

    match field_kind.inner() {
        // Well known types are `Message` types which get generated as `Option<T>`
        // Therefore we need to strip this option when its !nullable
        FieldKind::WellKnown(_) => {
            prost.field_attribute(key, "#[serde(serialize_with = \"::tinc::helpers::well_known::serialize\")]");
            prost.field_attribute(
                key,
                format!(
                    "#[serde(deserialize_with = \"{}\")]",
                    if is_optional && !nullable {
                        "::tinc::helpers::well_known::deserialize_non_optional"
                    } else {
                        "::tinc::helpers::well_known::deserialize"
                    }
                ),
            );
        }
        FieldKind::Enum(name) => {
            prost.field_attribute(
                key,
                format!(
                    "#[serde(with = \"::tinc::helpers::Enum::<{}>\")]",
                    get_common_import(key, name)
                ),
            );
        }
        _ if is_optional && !nullable => {
            prost.field_attribute(
                key,
                "#[serde(deserialize_with = \"::tinc::helpers::deserialize_non_null_option\")]",
            );
        }
        _ if is_optional && !omitable => {
            prost.field_attribute(
                key,
                "#[serde(deserialize_with = \"::tinc::helpers::deserialize_non_omitable\")]",
            );
        }
        _ => {}
    }

    if is_optional && (!nullable || !omitable) {
        field_kind = field_kind.strip_option();
        prost.field_attribute(key, "#[schemars(required)]");
    }

    if let Some(with) = schemars_with(field_kind, key) {
        prost.field_attribute(key, format!("#[schemars(with = \"{with}\")]"));
    }

    if nullable && !omitable {
        prost.field_attribute(key, "#[schemars(transform = ::tinc::helpers::schemars_non_omitable)]");
    }
}

fn rename_all_to_serde_rename_all(style: tinc_pb::RenameAll) -> Option<&'static str> {
    match style {
        tinc_pb::RenameAll::LowerCase => Some("lowercase"),
        tinc_pb::RenameAll::UpperCase => Some("uppercase"),
        tinc_pb::RenameAll::PascalCase => Some("PascalCase"),
        tinc_pb::RenameAll::CamelCase => Some("camelCase"),
        tinc_pb::RenameAll::SnakeCase => Some("snake_case"),
        tinc_pb::RenameAll::KebabCase => Some("kebab-case"),
        tinc_pb::RenameAll::ScreamingSnakeCase => Some("SCREAMING_SNAKE_CASE"),
        tinc_pb::RenameAll::ScreamingKebabCase => Some("SCREAMING-KEBAB-CASE"),
        tinc_pb::RenameAll::Unspecified => None,
    }
}

fn object_type_path(key: &str, package: &str) -> syn::Path {
    let mut key = key
        .strip_prefix(package)
        .unwrap_or(key)
        .split('.')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .collect::<Vec<_>>();

    let key_length = key.len();
    if key_length > 1 {
        for k in key[..key_length - 1].iter_mut() {
            *k = k
                .with_boundaries(&Boundary::digit_letter())
                .to_case(convert_case::Case::Snake);
        }

        key[key_length - 1] = key[key_length - 1].to_case(convert_case::Case::Pascal);
    }

    syn::parse_str::<syn::Path>(&key.join("::")).unwrap()
}

fn handle_message(
    message_key: &str,
    message: &MessageOpts,
    prost: &mut tonic_build::Config,
    _: &mut BTreeMap<String, Vec<syn::Item>>,
) -> anyhow::Result<()> {
    let message_custom_impl = message.opts.custom_impl.unwrap_or(false);
    for (oneof_name, oneof) in &message.oneofs {
        let oneof_key = format!("{message_key}.{oneof_name}");

        if !message_custom_impl {
            if let Some(rename) = &oneof.opts.rename {
                serde_rename(&oneof_key, rename, prost);
            }

            if !oneof.opts.nullable() {
                prost.enum_attribute(&oneof_key, "#[schemars(required)]");
            } else if !oneof.opts.omitable() {
                prost.enum_attribute(&oneof_key, "#[schemars(required)]");
                prost.enum_attribute(&oneof_key, "#[schemars(transform = ::tinc::helpers::schemars_non_omitable)]");
            }
        }

        if oneof.opts.custom_impl.unwrap_or(message_custom_impl) {
            continue;
        }

        enum_attributes(&oneof_key, prost, false);
        rename_all(&oneof_key, oneof.opts.rename_all, prost, true);

        match &oneof.opts.tagged {
            Some(Tagged {
                tag,
                content: Some(content),
            }) => {
                prost.enum_attribute(&oneof_key, format!("#[serde(tag = \"{tag}\", content = \"{content}\")]"));
            }
            Some(Tagged { tag, content: None }) => {
                prost.enum_attribute(&oneof_key, format!("#[serde(tag = \"{tag}\")]"));
            }
            None => {}
        }
    }

    if message_custom_impl {
        return Ok(());
    }

    message_attributes(message_key, prost);
    rename_all(message_key, message.opts.rename_all, prost, false);

    for (field_name, field) in &message.fields {
        if field
            .one_of
            .as_ref()
            .is_some_and(|oneof| message.oneofs.get(oneof).unwrap().opts.custom_impl.unwrap_or(false))
        {
            continue;
        }

        let name = field
            .opts
            .rename
            .as_ref()
            .or_else(|| message.opts.rename_all.is_none().then_some(&field.json_name));

        let field_key = if let Some(oneof) = &field.one_of {
            format!("{message_key}.{oneof}.{field_name}")
        } else {
            format!("{message_key}.{field_name}")
        };

        if let Some(name) = name {
            serde_rename(&field_key, name, prost);
        }

        with_attr(&field_key, &field.kind, field.nullable, field.omitable, prost);

        if field.omitable {
            field_omitable(&field_key, prost);
        }
        field_visibility(&field_key, prost, field.visibility);
    }

    Ok(())
}

fn handle_enum(
    enum_key: &str,
    enum_: &EnumOpts,
    prost: &mut tonic_build::Config,
    _: &mut BTreeMap<String, Vec<syn::Item>>,
) -> anyhow::Result<()> {
    if enum_.opts.custom_impl.unwrap_or(false) {
        return Ok(());
    }

    enum_attributes(enum_key, prost, enum_.opts.repr_enum.unwrap_or(false));
    if !enum_.opts.repr_enum() {
        // By default we use screaming_snake_case for enums
        let enum_rename_all = enum_.opts.rename_all.unwrap_or(tinc_pb::RenameAll::ScreamingSnakeCase as i32);
        rename_all(enum_key, Some(enum_rename_all), prost, true);
    }

    for (variant, variant_opts) in &enum_.variants {
        let variant_key = format!("{enum_key}.{variant}");

        if !enum_.opts.repr_enum() {
            if let Some(rename) = &variant_opts.opts.rename {
                serde_rename(&variant_key, rename, prost);
            }
        }

        field_visibility(&variant_key, prost, variant_opts.visibility);
    }

    Ok(())
}

fn parse_route(route: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut chars = route.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '{' {
            continue;
        }

        // Check if this is an escaped '{'
        if let Some(&'{') = chars.peek() {
            chars.next();
            continue;
        }

        let mut param = String::new();

        for c in chars.by_ref() {
            if c == '}' {
                params.push(param);
                break;
            }

            param.push(c);
        }
    }

    params
}

fn handle_service(
    service_key: &str,
    service: &ServiceOpts,
    extensions: &Extensions,
    prost: &mut tonic_build::Config,
    modules: &mut BTreeMap<String, Vec<syn::Item>>,
) -> anyhow::Result<()> {
    const EXPECT_FMT: &str = "service should be in the format <package>.<service>";

    let name = service_key
        .strip_prefix(service.package.as_str())
        .expect(EXPECT_FMT)
        .strip_prefix('.')
        .expect(EXPECT_FMT);

    let snake_name = name.to_case(convert_case::Case::Snake);
    let pascal_name = name.to_case(convert_case::Case::Pascal);

    fn ident_from_str(s: impl AsRef<str>) -> Ident {
        // if s is a keyword we need to escape it with r#
        if let Ok(ident) = syn::parse_str(s.as_ref()) {
            ident
        } else {
            Ident::new_raw(s.as_ref(), proc_macro2::Span::call_site())
        }
    }

    let service_trait = &ident_from_str(&pascal_name);
    let tinc_module_name = &ident_from_str(format!("{}_tinc", snake_name));
    let server_module_name = &ident_from_str(format!("{}_server", snake_name));
    let tinc_struct_name = &ident_from_str(format!("{}Tinc", pascal_name));

    let mut methods = Vec::new();
    let mut routes = Vec::new();

    for (name, method) in service.methods.iter() {
        for (idx, endpoint) in method.opts.iter().enumerate() {
            let (http_method_str, path) = match endpoint.method.as_ref() {
                Some(tinc_pb::http_endpoint_options::Method::Get(path)) => ("get", path),
                Some(tinc_pb::http_endpoint_options::Method::Post(path)) => ("post", path),
                Some(tinc_pb::http_endpoint_options::Method::Put(path)) => ("put", path),
                Some(tinc_pb::http_endpoint_options::Method::Delete(path)) => ("delete", path),
                Some(tinc_pb::http_endpoint_options::Method::Patch(path)) => ("patch", path),
                Some(tinc_pb::http_endpoint_options::Method::Custom(method)) => (method.method.as_str(), &method.path),
                _ => continue,
            };

            let path = if let Some(prefix) = &service.opts.prefix {
                format!(
                    "/{prefix}/{path}",
                    prefix = prefix.trim_end_matches('/'),
                    path = path.trim_start_matches('/')
                )
            } else {
                format!("/{path}", path = path.trim_start_matches('/'))
            };

            let service_method_name = ident_from_str(name.to_case(Case::Snake));
            let function_name = ident_from_str(format!("{name}_{http_method_str}_{idx}"));
            let http_method = ident_from_str(http_method_str);
            let params = parse_route(&path);

            enum IoOptions<'a> {
                Message(String, &'a MessageOpts),
                WellKnown(WellKnownType),
            }

            impl IoOptions<'_> {
                fn path(&self, package: &str) -> syn::Path {
                    match self {
                        IoOptions::Message(name, _) => {
                            let path = object_type_path(name.as_str(), package);
                            parse_quote! {
                                super::#path
                            }
                        }
                        IoOptions::WellKnown(well_known) => {
                            let well_known = ident_from_str(well_known.name());
                            syn::parse_quote! {
                                ::tinc::helpers::well_known::#well_known
                            }
                        }
                    }
                }

                fn has_content(&self, excluding: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
                    let excluding: Vec<_> = excluding.into_iter().collect();
                    match self {
                        IoOptions::Message(_, message) => message
                            .fields
                            .keys()
                            .any(|name| excluding.iter().all(|exclude| exclude.as_ref() != name)),
                        IoOptions::WellKnown(WellKnownType::Empty) => false,
                        IoOptions::WellKnown(_) => true,
                    }
                }

                fn has_fields(&self, fields: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
                    let fields: Vec<_> = fields.into_iter().collect();
                    if fields.is_empty() {
                        return true;
                    }

                    match self {
                        IoOptions::Message(_, message) => {
                            fields.into_iter().all(|field| message.fields.contains_key(field.as_ref()))
                        }
                        IoOptions::WellKnown(WellKnownType::Struct) => true, // maps always can take fields
                        IoOptions::WellKnown(_) => false,
                    }
                }

                fn field_type(&self, field: &str) -> Option<&FieldKind> {
                    match self {
                        IoOptions::Message(_, message) => message.fields.get(field).map(|field| &field.kind),
                        IoOptions::WellKnown(WellKnownType::Struct) => Some(&FieldKind::WellKnown(WellKnownType::Value)),
                        IoOptions::WellKnown(_) => None,
                    }
                }
            }

            fn parse_header(
                header: &tinc_pb::http_endpoint_options::Header,
                input_message: &IoOptions,
            ) -> anyhow::Result<proc_macro2::TokenStream> {
                use tinc_pb::http_endpoint_options::header;

                let header_name = header.name.as_str();
                let field_str = header.field.as_str();

                let field_type = input_message
                    .field_type(field_str)
                    .with_context(|| format!("header field {field_str} not found in input message"))?;

                let encoding = header.encoding.clone().unwrap_or_else(|| match field_type.strip_option() {
                    FieldKind::Map(_, _) | FieldKind::Message(_) | FieldKind::WellKnown(WellKnownType::Struct) => {
                        header::Encoding::ContentType(header::ContentType::FormUrlEncoded as i32)
                    }
                    _ => header::Encoding::ContentType(header::ContentType::Text as i32),
                });

                let header_value = match encoding {
                    header::Encoding::Delimiter(delimiter) => {
                        quote! {
                            ::tinc::helpers::header_decode::text(&parts.headers, #header_name, #field_str, ::core::option::Option::Some(#delimiter))
                        }
                    }
                    header::Encoding::ContentType(content_type) => {
                        let content_type = header::ContentType::try_from(content_type)
                            .with_context(|| format!("invalid header content type value: {content_type}"))?;

                        match content_type {
                            header::ContentType::FormUrlEncoded => {
                                quote! {
                                    ::tinc::helpers::header_decode::form_url_encoded(&parts.headers, #header_name, #field_str)
                                }
                            }
                            header::ContentType::Json => {
                                quote! {
                                    ::tinc::helpers::header_decode::json(&parts.headers, #header_name, #field_str)
                                }
                            }
                            header::ContentType::Unspecified | header::ContentType::Text => {
                                quote! {
                                    ::tinc::helpers::header_decode::text(&parts.headers, #header_name, #field_str, ::core::option::Option::None)
                                }
                            }
                        }
                    }
                };

                Ok(quote! {
                    input.merge(match #header_value {
                        Ok(input) => input,
                        Err(err) => return err,
                    });
                })
            }

            let input_message = match &method.input {
                MethodIo::Message(name) => {
                    let input_message = extensions.messages().get(name.as_str()).expect("input message not found");

                    IoOptions::Message(name.clone(), input_message)
                }
                MethodIo::WellKnown(well_known) => IoOptions::WellKnown(*well_known),
            };

            let endpoint_headers = endpoint
                .header
                .iter()
                .map(|header| parse_header(header, &input_message))
                .collect::<anyhow::Result<Vec<_>>>()?;

            anyhow::ensure!(
                input_message.has_fields(&params),
                "input message {} has missing fields: {:?}",
                name,
                params
            );

            let path_params = if !params.is_empty() {
                quote! {
                    match ::tinc::helpers::parse_path(&mut parts).await {
                        Ok(path_params) => {
                            input.merge(path_params);
                        },
                        Err(err) => return err,
                    }
                }
            } else {
                quote! {}
            };

            let is_get_or_delete = matches!(http_method_str, "get" | "delete");

            let use_query_string = endpoint.query_string.unwrap_or(is_get_or_delete);
            let query_string = if use_query_string {
                quote! {
                    match ::tinc::helpers::parse_query(&mut parts).await {
                        Ok(query_string) => {
                            input.merge(query_string);
                        },
                        Err(err) => return err,
                    }
                }
            } else {
                quote! {}
            };

            let use_request_body = endpoint.request_body.unwrap_or(!is_get_or_delete) && input_message.has_content(&params);
            let request_body = if use_request_body {
                let mut content_types = endpoint.content_type.clone();
                if content_types.is_empty() {
                    content_types.push(tinc_pb::http_endpoint_options::ContentType {
                        accept: vec!["application/json".to_string()],
                        content: None,
                        header: Vec::new(),
                        multipart: None,
                    });
                }

                let ct_idents = &content_types
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| ident_from_str(format!("ACCEPT_{idx}")))
                    .collect::<Vec<_>>();

                let const_cts = content_types
                    .iter()
                    .zip(ct_idents.iter())
                    .map(|(content_type, ct_ident)| {
                        let content_type =
                            content_type
                                .accept
                                .iter()
                                .map(|mime| {
                                    Ok(mime.parse::<mediatype::MediaTypeBuf>().context("invalid mime type")?.to_string())
                                })
                                .collect::<anyhow::Result<Vec<_>>>()?;

                        Ok(quote! {
                            static #ct_ident: ::std::sync::LazyLock<::tinc::reexports::headers_accept::Accept> = ::std::sync::LazyLock::new(|| {
                                ::std::iter::FromIterator::from_iter([
                                    #(
                                        ::tinc::reexports::mediatype::MediaTypeBuf::from_string(#content_type.to_owned()).expect("invalid mime type this is a bug, please report it")
                                    ),*
                                ])
                            });
                        })
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;

                let matchers = content_types
                    .iter()
                    .zip(ct_idents.iter())
                    .map(|(content_type, ct_ident)| {
                        let headers = content_type
                            .header
                            .iter()
                            .map(|header| parse_header(header, &input_message))
                            .collect::<anyhow::Result<Vec<_>>>()?;

                        let merge = content_type
                            .content
                            .as_ref()
                            .and_then(|content| match content {
                                tinc_pb::http_endpoint_options::content_type::Content::Body(field) => Some(quote! {
                                    input.merge(
                                        ::std::iter::once((::core::convert::Into::into(#field), body))
                                    )
                                }),
                                _ => None,
                            })
                            .unwrap_or_else(|| {
                                quote! {
                                    match body {
                                        ::tinc::value::Value::Object(object) => {
                                            input.merge(object);
                                        }
                                        _ => return ::tinc::helpers::bad_request_not_object(body),
                                    }
                                }
                            });

                        Ok(quote! {
                            if let Some(accept) = #ct_ident.negotiate([content_type]) {
                                #(#headers)*

                                match ::tinc::helpers::parse_body(accept, body).await {
                                    Ok(body) => #merge,
                                    Err(err) => return err,
                                }
                            }
                        })
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;

                quote! {
                    #(#const_cts)*

                    let content_type = match ::tinc::helpers::header_decode::content_type(&parts.headers) {
                        Ok(content_type) => content_type,
                        Err(err) => return err,
                    };

                    if let Some(content_type) = &content_type {
                        // the matchers expand to `else if ...` statements, so we start with if false {}
                        #(#matchers else)* {
                            return ::tinc::helpers::no_valid_content_type(content_type, &[
                                #(&*#ct_idents),*
                            ]);
                        }
                    }
                }
            } else {
                quote! {}
            };

            let input_path = input_message.path(service.package.as_str());

            let function_impl = quote! {
                let mut input = ::tinc::value::Object::new();

                #path_params

                #query_string

                #(#endpoint_headers)*

                #request_body

                let input: #input_path = match ::tinc::helpers::decode_input(input) {
                    Ok(input) => input,
                    Err(err) => return err,
                };

                let request = ::tinc::reexports::tonic::Request::from_parts(
                    ::tinc::reexports::tonic::metadata::MetadataMap::from_headers(parts.headers),
                    parts.extensions,
                    ::core::convert::Into::into(input),
                );

                let (metadata, body, extensions) = match service.inner.#service_method_name(request).await {
                    ::core::result::Result::Ok(response) => response.into_parts(),
                    ::core::result::Result::Err(status) => {
                        todo!("todo map errors: {:?}", status);
                    }
                };

                let mut response = ::tinc::reexports::axum::response::IntoResponse::into_response(
                    ::tinc::reexports::axum::extract::Json(body),
                );

                *response.headers_mut() = metadata.into_headers();
                *response.extensions_mut() = extensions;

                response
            };

            routes.push(quote! {
                .route(#path, ::tinc::reexports::axum::routing::#http_method(#function_name::<T>))
            });

            methods.push(quote! {
                #[allow(non_snake_case, unused_mut, dead_code, unused_variables)]
                async fn #function_name<T>(
                    ::tinc::reexports::axum::extract::State(service): ::tinc::reexports::axum::extract::State<#tinc_struct_name<T>>,
                    request: ::tinc::reexports::axum::extract::Request,
                ) -> ::tinc::reexports::axum::response::Response
                where
                    T: super::#server_module_name::#service_trait,
                {
                    let (mut parts, body) = ::tinc::reexports::axum::RequestExt::with_limited_body(request).into_parts();
                    #function_impl
                }
            });
        }
    }

    modules.entry(service.package.clone()).or_default().push(parse_quote! {
        /// This module was automatically generated by `tinc`.
        pub mod #tinc_module_name {
            /// A tinc service struct that exports gRPC routes via an axum router.
            pub struct #tinc_struct_name<T> {
                inner: ::std::sync::Arc<T>,
            }

            impl<T> #tinc_struct_name<T> {
                /// Create a new tinc service struct from a service implementation.
                pub fn new(inner: T) -> Self {
                    Self { inner: ::std::sync::Arc::new(inner) }
                }

                /// Create a new tinc service struct from an existing `Arc`.
                pub fn from_arc(inner: ::std::sync::Arc<T>) -> Self {
                    Self { inner }
                }
            }

            impl<T> ::std::clone::Clone for #tinc_struct_name<T> {
                fn clone(&self) -> Self {
                    Self { inner: self.inner.clone() }
                }
            }

            impl<T> ::std::fmt::Debug for #tinc_struct_name<T> {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, stringify!(#tinc_struct_name))
                }
            }

            impl<T> ::tinc::TincService for #tinc_struct_name<T>
            where
                T: super::#server_module_name::#service_trait
            {
                fn into_router(self) -> ::tinc::reexports::axum::Router {
                    #(#methods)*

                    ::tinc::reexports::axum::Router::new()
                        #(#routes)*
                        .with_state(self)
                }
            }
        }
    });

    Ok(())
}

pub fn generate_modules(
    extensions: &Extensions,
    prost: &mut tonic_build::Config,
) -> anyhow::Result<BTreeMap<String, Vec<syn::Item>>> {
    let mut modules = BTreeMap::new();

    extensions
        .messages()
        .iter()
        .try_for_each(|(key, message)| handle_message(key, message, prost, &mut modules))?;

    extensions
        .enums()
        .iter()
        .try_for_each(|(key, enum_)| handle_enum(key, enum_, prost, &mut modules))?;

    extensions
        .services()
        .iter()
        .try_for_each(|(key, service)| handle_service(key, service, extensions, prost, &mut modules))?;

    Ok(modules)
}
