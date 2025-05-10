use std::collections::BTreeMap;

use anyhow::Context;
use base64::Engine;
use tinc_cel::{CelValue, NumberTy};

use super::optimize::optimize_json_schema;
use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerTarget, ConstantCompiledExpr};
use crate::codegen::cel::{CelExpression, CelExpressions, functions};
use crate::types::{
    ProtoMessageType, ProtoModifiedValueType, ProtoType, ProtoTypeRegistry, ProtoValueType, ProtoWellKnownType,
};

fn cel_to_json(cel: &CelValue<'static>, type_registry: &ProtoTypeRegistry) -> anyhow::Result<serde_json::Value> {
    match cel {
        CelValue::Null => Ok(serde_json::Value::Null),
        CelValue::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        CelValue::Map(map) => Ok(serde_json::Value::Object(
            map.iter()
                .map(|(key, value)| {
                    if let CelValue::String(key) = key {
                        Ok((key.to_string(), cel_to_json(value, type_registry)?))
                    } else {
                        anyhow::bail!("map keys must be a string")
                    }
                })
                .collect::<anyhow::Result<_>>()?,
        )),
        CelValue::List(list) => Ok(serde_json::Value::Array(
            list.iter()
                .map(|i| cel_to_json(i, type_registry))
                .collect::<anyhow::Result<_>>()?,
        )),
        CelValue::String(s) => Ok(serde_json::Value::String(s.to_string())),
        CelValue::Number(NumberTy::F64(f)) => Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(*f).context("f64 is not a valid float")?,
        )),
        CelValue::Number(NumberTy::I64(i)) => Ok(serde_json::Value::Number(
            serde_json::Number::from_i128(*i as i128).context("i64 is not a valid int")?,
        )),
        CelValue::Number(NumberTy::U64(u)) => Ok(serde_json::Value::Number(
            serde_json::Number::from_u128(*u as u128).context("u64 is not a valid uint")?,
        )),
        CelValue::Duration(duration) => Ok(serde_json::Value::String(duration.to_string())),
        CelValue::Timestamp(timestamp) => Ok(serde_json::Value::String(timestamp.to_rfc3339())),
        CelValue::Bytes(bytes) => Ok(serde_json::Value::String(
            base64::engine::general_purpose::STANDARD.encode(bytes),
        )),
        CelValue::Enum(cel_enum) => {
            let enum_ty = type_registry
                .get_enum(&cel_enum.tag)
                .with_context(|| format!("couldnt find enum {}", cel_enum.tag.as_ref()))?;
            if enum_ty.options.repr_enum {
                Ok(serde_json::Value::from(cel_enum.value))
            } else {
                let variant = enum_ty
                    .variants
                    .values()
                    .find(|v| v.value == cel_enum.value)
                    .with_context(|| format!("{} has no value for {}", cel_enum.tag.as_ref(), cel_enum.value))?;
                Ok(serde_json::Value::from(variant.options.serde_name.clone()))
            }
        }
    }
}

#[derive(Default, Debug, serde_derive::Serialize)]
pub(crate) struct SchemaRegistry {
    items: BTreeMap<String, serde_json::Value>,
}

impl std::ops::Deref for SchemaRegistry {
    type Target = BTreeMap<String, serde_json::Value>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl std::ops::DerefMut for SchemaRegistry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

fn parse_resolve(compiler: &Compiler, expr: &str) -> anyhow::Result<CelValue<'static>> {
    let expr = cel_parser::parse(expr).context("parse")?;
    let resolved = compiler.resolve(&expr).context("resolve")?;
    match resolved {
        CompiledExpr::Constant(ConstantCompiledExpr { value }) => Ok(value),
        CompiledExpr::Runtime(_) => anyhow::bail!("expression needs runtime evaluation"),
    }
}

fn handle_expr(mut ctx: Compiler, ty: &ProtoType, expr: &CelExpression) -> anyhow::Result<Vec<serde_json::Value>> {
    ctx.set_target(CompilerTarget::Serde);

    if let Some(this) = expr.this.clone() {
        ctx.add_variable("this", CompiledExpr::constant(this));
    }

    match ty {
        ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Enum(path)))
        | ProtoType::Value(ProtoValueType::Enum(path)) => {
            ctx.register_function(functions::Enum(Some(path.clone())));
        }
        _ => {}
    }

    let mut schemas = Vec::new();
    for schema in &expr.jsonschemas {
        let value = parse_resolve(&ctx, schema)?;
        let value = cel_to_json(&value, ctx.registry())?;
        if !value.is_null() {
            schemas.push(value);
        }
    }

    Ok(schemas)
}

#[derive(Debug)]
pub(crate) enum ExcludePaths {
    True,
    Child(BTreeMap<String, ExcludePaths>),
}

pub(super) fn exclude_path(paths: &mut BTreeMap<String, ExcludePaths>, path: &str) -> anyhow::Result<()> {
    let mut parts = path.split('.').peekable();
    let first_part = parts.next().expect("parts empty").to_owned();

    // Start with the first part of the path
    let mut current_map = paths.entry(first_part).or_insert(if parts.peek().is_none() {
        ExcludePaths::True
    } else {
        ExcludePaths::Child(BTreeMap::new())
    });

    // Iterate over the remaining parts of the path
    while let Some(part) = parts.next() {
        match current_map {
            ExcludePaths::True => anyhow::bail!("duplicate path: {path}"),
            ExcludePaths::Child(map) => {
                current_map = map.entry(part.to_owned()).or_insert(if parts.peek().is_none() {
                    ExcludePaths::True
                } else {
                    ExcludePaths::Child(BTreeMap::new())
                });
            }
        }
    }

    anyhow::ensure!(matches!(current_map, ExcludePaths::True), "duplicate path: {path}");

    Ok(())
}

pub(super) fn generate_query_parameter(
    type_registry: &ProtoTypeRegistry,
    schema_registry: &mut SchemaRegistry,
    ty: &ProtoMessageType,
    exclude_paths: &BTreeMap<String, ExcludePaths>,
) -> anyhow::Result<Vec<serde_json::Value>> {
    let mut params = Vec::new();

    for (name, field) in &ty.fields {
        let exclude_paths = match exclude_paths.get(name) {
            Some(ExcludePaths::True) => continue,
            Some(ExcludePaths::Child(child)) => Some(child),
            None => None,
        };
        params.push(serde_json::json!({
            "name": &field.options.serde_name,
            "required": !field.options.serde_omittable.is_true(),
            "explode": true,
            "style": "deepObject",
            "schema": generate_optimized(type_registry, schema_registry, field.ty.clone(), &field.options.cel_exprs, exclude_paths.unwrap_or(&BTreeMap::new()), GenerateDirection::Input, BytesEncoding::Base64)?,
        }))
    }

    Ok(params)
}

pub(super) fn generate_path_parameter(
    type_registry: &ProtoTypeRegistry,
    schema_registry: &mut SchemaRegistry,
    paths: &BTreeMap<String, (ProtoValueType, CelExpressions)>,
) -> anyhow::Result<Vec<serde_json::Value>> {
    let mut params = Vec::new();

    for (path, (ty, cel)) in paths {
        params.push(serde_json::json!({
            "name": path,
            "required": true,
            "schema": generate_optimized(type_registry, schema_registry, ProtoType::Value(ty.clone()), cel, &BTreeMap::new(), GenerateDirection::Input, BytesEncoding::Base64)?,
        }))
    }

    Ok(params)
}

#[derive(Debug, Clone, Copy)]
pub(super) enum BytesEncoding {
    Base64,
    Binary,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum GenerateDirection {
    Input,
    Output,
}

#[inline(always)]
const fn noop<T>(t: T) -> T {
    t
}

pub(super) fn generate_optimized(
    type_registry: &ProtoTypeRegistry,
    schema_registry: &mut SchemaRegistry,
    ty: ProtoType,
    cel: &CelExpressions,
    exclude_paths: &BTreeMap<String, ExcludePaths>,
    direction: GenerateDirection,
    bytes: BytesEncoding,
) -> anyhow::Result<serde_json::Value> {
    generate(type_registry, schema_registry, ty, cel, exclude_paths, direction, bytes).map(|v| optimize_json_schema([v]))
}

fn generate(
    type_registry: &ProtoTypeRegistry,
    schema_registry: &mut SchemaRegistry,
    ty: ProtoType,
    cel: &CelExpressions,
    exclude_paths: &BTreeMap<String, ExcludePaths>,
    direction: GenerateDirection,
    bytes: BytesEncoding,
) -> anyhow::Result<serde_json::Value> {
    let mut schemas = Vec::new();

    let compiler = Compiler::new(type_registry);
    if !matches!(ty, ProtoType::Modified(ProtoModifiedValueType::Optional(_))) {
        schemas.reserve(cel.field.len());
        for expr in &cel.field {
            schemas.extend(handle_expr(compiler.child(), &ty, expr)?);
        }
    } else {
        schemas.reserve(1);
    }

    schemas.push(match ty {
        ProtoType::Modified(ProtoModifiedValueType::Map(key, value)) => serde_json::json!({
            "type": "object",
            "additionalProperties": true,
            "propertyNames": match key {
                ProtoValueType::String => {
                    let mut schemas = Vec::with_capacity(1 + cel.map_key.len());

                    for expr in &cel.map_key {
                        schemas.extend(handle_expr(compiler.child(), &ProtoType::Value(key.clone()), expr)?);
                    }

                    schemas.push(serde_json::json!({
                        "type": "string",
                    }));


                    serde_json::json!({
                        "allOf": schemas,
                    })
                }
                ty @ (ProtoValueType::Int32 | ProtoValueType::Int64) => serde_json::json!({
                    "type": "string",
                    "title": if matches!(ty, ProtoValueType::Int32) {
                        "int32"
                    } else {
                        "int64"
                    },
                    "pattern": "^-?[0-9]+$",
                }),
                ty @ (ProtoValueType::UInt32 | ProtoValueType::UInt64) => serde_json::json!({
                    "type": "string",
                    "title": if matches!(ty, ProtoValueType::UInt32) {
                        "uint32"
                    } else {
                        "uint64"
                    },
                    "pattern": "^[0-9]+$",
                }),
                ProtoValueType::Bool => serde_json::json!({
                    "type": "string",
                    "title": "bool",
                    "enum": ["true", "false"],
                }),
                _ => serde_json::json!({
                    "type": "string",
                }),
            },
            "additionalProperties": noop({
                let mut schemas = Vec::with_capacity(1 + cel.map_value.len());
                for expr in &cel.map_value {
                    schemas.extend(handle_expr(compiler.child(), &ProtoType::Value(value.clone()), expr)?);
                }

                schemas.push(generate(
                    type_registry,
                    schema_registry,
                    ProtoType::Value(value),
                    &CelExpressions::default(),
                    &BTreeMap::new(),
                    direction,
                    bytes,
                )?);

                serde_json::json!({
                    "allOf": schemas,
                })
            }),
        }),
        ProtoType::Modified(ProtoModifiedValueType::Repeated(item)) => serde_json::json!({
            "type": "array",
            "items": generate(type_registry, schema_registry, ProtoType::Value(item), cel, exclude_paths, direction, bytes)?,
        }),
        ProtoType::Modified(ProtoModifiedValueType::OneOf(oneof)) => {
            serde_json::json!({
                "type": "object",
                "title": oneof.full_name.as_ref(),
                "oneOf": if let Some(tagged) = oneof.options.tagged {
                    oneof
                        .fields
                        .into_iter()
                        .filter(|(_, field)| match direction {
                            GenerateDirection::Input => field.options.visibility.has_input(),
                            GenerateDirection::Output => field.options.visibility.has_output(),
                        })
                        .map(|(name, field)| {
                            let ty = generate(
                                type_registry,
                                schema_registry,
                                ProtoType::Value(field.ty),
                                &field.options.cel_exprs,
                                &BTreeMap::new(),
                                direction,
                                bytes,
                            )?;

                            anyhow::Ok(serde_json::json!({
                                "type": "object",
                                "title": name,
                                "description": field.comments.to_string(),
                                "properties": {
                                    tagged.tag.as_str(): {
                                        "type": "string",
                                        "const": field.options.serde_name,
                                    },
                                    tagged.content.as_str(): ty,
                                },
                                "unevaluatedProperties": false,
                            }))
                        })
                        .collect::<anyhow::Result<Vec<_>>>()?
                } else {
                    oneof
                        .fields
                        .into_iter()
                        .filter(|(_, field)| match direction {
                            GenerateDirection::Input => field.options.visibility.has_input(),
                            GenerateDirection::Output => field.options.visibility.has_output(),
                        })
                        .map(|(name, field)| {
                            let ty = generate(
                                type_registry,
                                schema_registry,
                                ProtoType::Value(field.ty),
                                &field.options.cel_exprs,
                                &BTreeMap::new(),
                                direction,
                                bytes,
                            )?;

                            anyhow::Ok(serde_json::json!({
                                "type": "object",
                                "title": name,
                                "description": field.comments.to_string(),
                                "properties": {
                                    field.options.serde_name: ty,
                                },
                                "unevaluatedProperties": false,
                            }))
                        })
                        .collect::<anyhow::Result<Vec<_>>>()?
                },
                "unevaluatedProperties": false,
            })
        }
        ProtoType::Modified(ProtoModifiedValueType::Optional(value)) => serde_json::json!({
            "oneOf": [
                { "type": null },
                generate(type_registry, schema_registry, ProtoType::Value(value), cel, exclude_paths, direction, bytes)?,
            ],
        }),
        ProtoType::Value(ProtoValueType::Bool) => serde_json::json!({
            "type": "boolean",
        }),
        ProtoType::Value(ProtoValueType::Bytes) => serde_json::json!({
            "type": "string",
            "title": "bytes",
            "contentEncoding": match bytes {
                BytesEncoding::Base64 => "base64",
                BytesEncoding::Binary => "binary",
            },
        }),
        ProtoType::Value(ProtoValueType::Double | ProtoValueType::Float) => serde_json::json!({
            "type": "number",
        }),
        ProtoType::Value(ProtoValueType::Int32) => serde_json::json!({
            "type": "integer",
            "title": "int32",
            "minimum": i32::MIN,
            "maximum": i32::MAX,
        }),
        ProtoType::Value(ProtoValueType::UInt32) => serde_json::json!({
            "type": "integer",
            "title": "uint32",
            "minimum": u32::MIN,
            "maximum": u32::MAX,
        }),
        ProtoType::Value(ProtoValueType::Int64) => serde_json::json!({
            "type": "integer",
            "title": "int64",
            "minimum": i64::MIN,
            "maximum": i64::MAX,
        }),
        ProtoType::Value(ProtoValueType::UInt64) => serde_json::json!({
            "type": "integer",
            "title": "uint64",
            "minimum": u64::MIN,
            "maximum": u64::MAX,
        }),
        ProtoType::Value(ProtoValueType::String) => serde_json::json!({
            "type": "string",
        }),
        ProtoType::Value(ProtoValueType::Enum(enum_path)) => {
            if !schema_registry.contains_key(enum_path.as_ref()) {
                let ety = type_registry
                    .get_enum(&enum_path)
                    .with_context(|| format!("missing enum: {enum_path}"))?;
                let ty = if ety.options.repr_enum { "intger" } else { "string" };
                let values = ety
                    .variants
                    .values()
                    .filter(|v| match direction {
                        GenerateDirection::Input => v.options.visibility.has_input(),
                        GenerateDirection::Output => v.options.visibility.has_output(),
                    })
                    .map(|v| {
                        if ety.options.repr_enum {
                            serde_json::Value::from(v.value)
                        } else {
                            serde_json::Value::from(v.options.serde_name.clone())
                        }
                    })
                    .collect::<Vec<_>>();

                schema_registry.insert(
                    format!("{direction:?}.{enum_path}"),
                    serde_json::json!({
                        "type": ty,
                        "title": enum_path.as_ref(),
                        "description": ety.comments.to_string(),
                        "enum": values,
                    }),
                );
            }

            serde_json::json!({
                "$ref": format!("#/components/schemas/{direction:?}.{enum_path}"),
            })
        }
        ProtoType::Value(ProtoValueType::Message(message_path)) => {
            let message_ty = type_registry
                .get_message(&message_path)
                .with_context(|| format!("missing message: {message_path}"))?;

            if !schema_registry.contains_key(message_path.as_ref()) || !exclude_paths.is_empty() {
                if exclude_paths.is_empty() {
                    schema_registry.insert(format!("{direction:?}.{message_path}"), serde_json::Value::Null);
                }
                let mut properties = serde_json::Map::new();
                let mut required = Vec::new();
                let mut schemas = Vec::with_capacity(1);
                for (name, field) in message_ty.fields.iter().filter(|(_, field)| match direction {
                    GenerateDirection::Input => field.options.visibility.has_input(),
                    GenerateDirection::Output => field.options.visibility.has_output(),
                }) {
                    let exclude_paths = match exclude_paths.get(name) {
                        Some(ExcludePaths::True) => continue,
                        Some(ExcludePaths::Child(child)) => Some(child),
                        None => None,
                    };
                    if !field.options.serde_omittable.is_true() {
                        required.push(field.options.serde_name.clone());
                    }

                    let ty = match (!field.options.nullable || field.options.flatten, &field.ty) {
                        (true, ProtoType::Modified(ProtoModifiedValueType::Optional(ty))) => ProtoType::Value(ty.clone()),
                        _ => field.ty.clone(),
                    };

                    let field_schema = generate(
                        type_registry,
                        schema_registry,
                        ty,
                        &field.options.cel_exprs,
                        exclude_paths.unwrap_or(&BTreeMap::new()),
                        direction,
                        bytes,
                    )?;

                    if field.options.flatten {
                        schemas.push(field_schema);
                    } else {
                        let schema = if field.options.nullable
                            && !matches!(&field.ty, ProtoType::Modified(ProtoModifiedValueType::Optional(_)))
                        {
                            serde_json::json!({
                                "oneOf": [
                                    { "type": null },
                                    field_schema,
                                ]
                            })
                        } else {
                            field_schema
                        };

                        properties.insert(
                            field.options.serde_name.clone(),
                            serde_json::json!({
                                "allOf": [
                                    schema,
                                    serde_json::json!({
                                        "description": field.comments.to_string(),
                                    }),
                                ],
                            }),
                        );
                    }
                }

                schemas.push(serde_json::json!({
                    "type": "object",
                    "title": message_path.as_ref(),
                    "description": message_ty.comments.to_string(),
                    "properties": properties,
                    "required": required,
                    "unevaluatedProperties": false,
                }));

                if exclude_paths.is_empty() {
                    schema_registry.insert(format!("{direction:?}.{message_path}"), optimize_json_schema(schemas));
                    serde_json::json!({
                        "$ref": format!("#/components/schemas/{direction:?}.{message_path}"),
                    })
                } else {
                    serde_json::json!({
                        "allOf": schemas,
                    })
                }
            } else {
                serde_json::json!({
                    "$ref": format!("#/components/schemas/{direction:?}.{message_path}"),
                })
            }
        }
        ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Timestamp)) => serde_json::json!({
            "type": "string",
            "format": "date-time",
        }),
        ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Duration)) => serde_json::json!({
            "type": "string",
            "format": "duration",
        }),
        ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Empty)) => serde_json::json!({
            "type": "object",
            "unevaluatedProperties": false,
        }),
        ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::ListValue)) => serde_json::json!({
            "type": "array",
        }),
        ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Value)) => serde_json::json!({
            "type": ["null", "boolean", "object", "array", "number", "string"],
        }),
        ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Struct)) => serde_json::json!({
            "type": "object",
        }),
        ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Any)) => serde_json::json!({
            "type": "object",
            "properties": {
                "@type": {
                    "type": "string"
                }
            }
        }),
    });

    Ok(serde_json::json!({
        "allOf": schemas,
    }))
}
