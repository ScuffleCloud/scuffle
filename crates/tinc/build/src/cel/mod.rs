use std::collections::{BTreeMap, HashMap};

use anyhow::Context;
use functions::Function;
use quote::quote;

use crate::extensions::Extension;

pub mod codegen;
pub mod compiler;
pub mod functions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum CelInput {
    MapKey,
    MapValue,
    RepeatedItem,
    Root,
}

#[derive(Debug, Clone)]
pub struct MessageFormat {
    pub format: String,
    pub args: Vec<cel_parser::Expression>,
}

impl MessageFormat {
    pub fn new(msg: &str, ctx: &cel_interpreter::Context, has_this: bool) -> anyhow::Result<Self> {
        let fmt =
            runtime_format::ParsedFmt::new(msg).map_err(|err| anyhow::anyhow!("failed to parse message format: {err}"))?;

        let mut runtime_args = Vec::new();
        let mut compile_time_args = HashMap::new();

        // each key itself a cel expression
        for key in fmt.keys() {
            let expr = cel_parser::parse(key).context("failed to parse cel expression")?;
            match preevaluate_expression(expr, ctx, has_this)? {
                ConstantOrExpression::Constant(value) => {
                    compile_time_args.insert(key, value_to_str(&value).to_string());
                }
                ConstantOrExpression::Expression(expr) => {
                    compile_time_args.insert(key, format!("{{arg_{}}}", runtime_args.len()));
                    runtime_args.push(expr);
                }
            }
        }

        Ok(Self {
            format: fmt.with_args(&compile_time_args).to_string(),
            args: runtime_args,
        })
    }
}

fn prost_to_cel(v: prost_reflect::Value) -> anyhow::Result<cel_interpreter::Value> {
    match v {
        prost_reflect::Value::String(s) => Ok(cel_interpreter::Value::String(s.into())),
        prost_reflect::Value::Message(_) => anyhow::bail!("message not supported"),
        prost_reflect::Value::EnumNumber(_) => anyhow::bail!("enum not supported"),
        prost_reflect::Value::Bool(b) => Ok(cel_interpreter::Value::Bool(b)),
        prost_reflect::Value::I32(i) => Ok(cel_interpreter::Value::Int(i as i64)),
        prost_reflect::Value::I64(i) => Ok(cel_interpreter::Value::Int(i)),
        prost_reflect::Value::U32(i) => Ok(cel_interpreter::Value::UInt(i as u64)),
        prost_reflect::Value::U64(i) => Ok(cel_interpreter::Value::UInt(i)),
        prost_reflect::Value::Bytes(b) => Ok(cel_interpreter::Value::Bytes(b.to_vec().into())),
        prost_reflect::Value::F32(f) => Ok(cel_interpreter::Value::Float(f as f64)),
        prost_reflect::Value::F64(f) => Ok(cel_interpreter::Value::Float(f)),
        prost_reflect::Value::List(list) => list
            .into_iter()
            .map(prost_to_cel)
            .collect::<anyhow::Result<Vec<_>>>()
            .map(Into::into)
            .map(cel_interpreter::Value::List),
        prost_reflect::Value::Map(map) => map
            .into_iter()
            .map(|(k, v)| {
                let k = match k {
                    prost_reflect::MapKey::Bool(b) => cel_interpreter::objects::Key::Bool(b),
                    prost_reflect::MapKey::I32(i) => cel_interpreter::objects::Key::Int(i as i64),
                    prost_reflect::MapKey::I64(i) => cel_interpreter::objects::Key::Int(i),
                    prost_reflect::MapKey::U32(i) => cel_interpreter::objects::Key::Uint(i as u64),
                    prost_reflect::MapKey::U64(i) => cel_interpreter::objects::Key::Uint(i),
                    prost_reflect::MapKey::String(s) => cel_interpreter::objects::Key::String(s.into()),
                };

                let v = prost_to_cel(v)?;
                Ok((k, v))
            })
            .collect::<anyhow::Result<HashMap<_, _>>>()
            .map(|map| cel_interpreter::Value::Map(cel_interpreter::objects::Map { map: map.into() })),
    }
}

struct FuncFmtter<F: Fn(&mut std::fmt::Formatter) -> std::fmt::Result>(F);

impl<F> std::fmt::Display for FuncFmtter<F>
where
    F: Fn(&mut std::fmt::Formatter) -> std::fmt::Result,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.0)(f)
    }
}

impl<F> std::fmt::Debug for FuncFmtter<F>
where
    F: Fn(&mut std::fmt::Formatter) -> std::fmt::Result,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.0)(f)
    }
}

fn value_to_str(v: &cel_interpreter::Value) -> impl std::fmt::Display + std::fmt::Debug {
    FuncFmtter(move |fmt| match v {
        cel_interpreter::Value::Bool(b) => std::fmt::Display::fmt(b, fmt),
        cel_interpreter::Value::Int(i) => std::fmt::Display::fmt(i, fmt),
        cel_interpreter::Value::UInt(i) => std::fmt::Display::fmt(i, fmt),
        cel_interpreter::Value::Float(f) => std::fmt::Display::fmt(f, fmt),
        cel_interpreter::Value::String(s) => std::fmt::Display::fmt(s, fmt),
        cel_interpreter::Value::Bytes(b) => {
            let lit = syn::LitByteStr::new(b, proc_macro2::Span::call_site());
            std::fmt::Display::fmt(&quote! { #lit }, fmt)
        }
        cel_interpreter::Value::List(list) => fmt.debug_list().entries(list.iter().map(value_to_str)).finish(),
        cel_interpreter::Value::Function(name, arg) => panic!(),
        cel_interpreter::Value::Map(map) => {
            let mut fmt = fmt.debug_map();
            for (k, v) in map.map.iter() {
                fmt.entry(
                    match k {
                        cel_interpreter::objects::Key::Bool(b) => b,
                        cel_interpreter::objects::Key::Int(i) => i,
                        cel_interpreter::objects::Key::Uint(i) => i,
                        cel_interpreter::objects::Key::String(s) => s,
                    },
                    &value_to_str(v),
                );
            }
            fmt.finish()
        }
        cel_interpreter::Value::Null => fmt.write_str("null"),
    })
}
enum ConstantOrExpression {
    // we have enough data now to evaluate the expression
    Constant(cel_interpreter::Value),
    // we need to evaluate the expression at runtime but this is the most we can do
    Expression(cel_parser::Expression),
}

fn cel_value_to_expr(value: cel_interpreter::Value) -> cel_parser::Expression {
    match value {
        cel_interpreter::Value::Bool(b) => cel_parser::Expression::Atom(cel_parser::Atom::Bool(b)),
        cel_interpreter::Value::Int(i) => cel_parser::Expression::Atom(cel_parser::Atom::Int(i)),
        cel_interpreter::Value::UInt(i) => cel_parser::Expression::Atom(cel_parser::Atom::UInt(i)),
        cel_interpreter::Value::Float(f) => cel_parser::Expression::Atom(cel_parser::Atom::Float(f)),
        cel_interpreter::Value::String(s) => cel_parser::Expression::Atom(cel_parser::Atom::String(s)),
        cel_interpreter::Value::Bytes(b) => cel_parser::Expression::Atom(cel_parser::Atom::Bytes(b)),
        cel_interpreter::Value::List(list) => {
            cel_parser::Expression::List(list.iter().cloned().map(cel_value_to_expr).collect())
        }
        cel_interpreter::Value::Function(name, arg) => {
            unreachable!("value should not be function at this point: {name} on {arg:?}")
        }
        cel_interpreter::Value::Map(map) => cel_parser::Expression::Map({
            let map = BTreeMap::from_iter(map.map.iter());
            map.into_iter()
                .map(|(k, v)| (cel_value_to_expr(k.clone().into()), cel_value_to_expr(v.clone())))
                .collect()
        }),
        cel_interpreter::Value::Null => cel_parser::Expression::Atom(cel_parser::Atom::Null),
    }
}

impl ConstantOrExpression {
    fn into_expr(self) -> cel_parser::Expression {
        match self {
            ConstantOrExpression::Constant(value) => cel_value_to_expr(value),
            ConstantOrExpression::Expression(expr) => expr,
        }
    }
}

fn preevaluate_expression(
    expr: cel_parser::Expression,
    ctx: &cel_interpreter::Context,
    has_this: bool,
) -> anyhow::Result<ConstantOrExpression> {
    let mut requires_runtime = false;
    for variable in expr.references().variables() {
        match variable {
            "input" => requires_runtime = true,
            "this" if has_this => {}
            _ => anyhow::bail!("unknown variable in expression: {variable}"),
        }
    }

    if !requires_runtime {
        return ctx
            .resolve(&expr)
            .context("failed to resolve expression")
            .map(ConstantOrExpression::Constant);
    }

    let result = match expr {
        cel_parser::Expression::Arithmetic(left, op, right) => cel_parser::Expression::Arithmetic(
            Box::new(preevaluate_expression(*left, ctx, has_this)?.into_expr()),
            op,
            Box::new(preevaluate_expression(*right, ctx, has_this)?.into_expr()),
        ),
        cel_parser::Expression::Relation(left, op, right) => cel_parser::Expression::Relation(
            Box::new(preevaluate_expression(*left, ctx, has_this)?.into_expr()),
            op,
            Box::new(preevaluate_expression(*right, ctx, has_this)?.into_expr()),
        ),
        cel_parser::Expression::Ternary(cmp, t, f) => cel_parser::Expression::Ternary(
            Box::new(preevaluate_expression(*cmp, ctx, has_this)?.into_expr()),
            Box::new(preevaluate_expression(*t, ctx, has_this)?.into_expr()),
            Box::new(preevaluate_expression(*f, ctx, has_this)?.into_expr()),
        ),
        cel_parser::Expression::Or(left, right) => cel_parser::Expression::Or(
            Box::new(preevaluate_expression(*left, ctx, has_this)?.into_expr()),
            Box::new(preevaluate_expression(*right, ctx, has_this)?.into_expr()),
        ),
        cel_parser::Expression::And(left, right) => cel_parser::Expression::And(
            Box::new(preevaluate_expression(*left, ctx, has_this)?.into_expr()),
            Box::new(preevaluate_expression(*right, ctx, has_this)?.into_expr()),
        ),
        cel_parser::Expression::Unary(op, expr) => {
            cel_parser::Expression::Unary(op, Box::new(preevaluate_expression(*expr, ctx, has_this)?.into_expr()))
        }
        cel_parser::Expression::Member(expr, member) => cel_parser::Expression::Member(
            Box::new(preevaluate_expression(*expr, ctx, has_this)?.into_expr()),
            Box::new(match *member {
                cel_parser::Member::Attribute(attr) => cel_parser::Member::Attribute(attr),
                cel_parser::Member::Index(index) => {
                    cel_parser::Member::Index(Box::new(preevaluate_expression(*index, ctx, has_this)?.into_expr()))
                }
                cel_parser::Member::Fields(fields) => cel_parser::Member::Fields(
                    fields
                        .into_iter()
                        .map(|(key, value)| {
                            let value = preevaluate_expression(value, ctx, has_this)?.into_expr();
                            Ok((key, value))
                        })
                        .collect::<anyhow::Result<Vec<_>>>()?,
                ),
            }),
        ),
        cel_parser::Expression::FunctionCall(func, this, args) => cel_parser::Expression::FunctionCall(
            func,
            if let Some(this) = this {
                Some(Box::new(preevaluate_expression(*this, ctx, has_this)?.into_expr()))
            } else {
                None
            },
            args.into_iter()
                .map(|arg| preevaluate_expression(arg, ctx, has_this).map(|value| value.into_expr()))
                .collect::<anyhow::Result<Vec<_>>>()?,
        ),
        cel_parser::Expression::List(exprs) => cel_parser::Expression::List(
            exprs
                .into_iter()
                .map(|expr| preevaluate_expression(expr, ctx, has_this).map(|value| value.into_expr()))
                .collect::<anyhow::Result<Vec<_>>>()?,
        ),
        cel_parser::Expression::Map(pairs) => cel_parser::Expression::Map(
            pairs
                .into_iter()
                .map(|(key, value)| {
                    let key = preevaluate_expression(key, ctx, has_this)?.into_expr();
                    let value = preevaluate_expression(value, ctx, has_this)?.into_expr();
                    Ok((key, value))
                })
                .collect::<anyhow::Result<Vec<_>>>()?,
        ),
        cel_parser::Expression::Atom(atom) => cel_parser::Expression::Atom(atom),
        cel_parser::Expression::Ident(ident) => cel_parser::Expression::Ident(ident),
    };

    Ok(ConstantOrExpression::Expression(result))
}

#[derive(Debug, Clone)]
pub struct CelExpression {
    pub expression: cel_parser::Expression,
    pub message: MessageFormat,
    pub json_schemas: Vec<serde_json::Value>,
}

impl CelExpression {
    pub fn new(pb: &tinc_pb::CelExpression, this: Option<&prost_reflect::Value>) -> anyhow::Result<Self> {
        let mut ctx = cel_interpreter::Context::empty();
        if let Some(this) = this {
            ctx.add_variable_from_value("this", prost_to_cel(this.clone())?);
        }

        functions::Contains::add_to_ctx(&mut ctx);
        functions::Size::add_to_ctx(&mut ctx);

        let message = MessageFormat::new(&pb.message, &ctx, this.is_some()).context("failed to create message format")?;

        let expression = cel_parser::parse(&pb.expression).context("failed to parse cel expression")?;

        let expression = preevaluate_expression(expression, &ctx, this.is_some())
            .context("failed to pre-evaluate expression")?
            .into_expr();

        let mut json_schemas = Vec::new();
        for schema in &pb.jsonschemas {
            let expr = cel_parser::parse(schema).context("failed to parse cel expression")?;
            let result = ctx.resolve(&expr).context("failed to resolve expression")?;
            let json = result
                .json()
                .map_err(|err| anyhow::anyhow!("failed to convert expression to json: {err}"))?;
            json_schemas.push(json);
        }

        let result = Ok(Self {
            expression,
            message,
            json_schemas,
        });

        result
    }
}

pub fn gather_cel_expressions(
    extension: &Extension<tinc_pb::PredefinedConstraint>,
    field_options: &prost_reflect::DynamicMessage,
) -> anyhow::Result<BTreeMap<CelInput, Vec<CelExpression>>> {
    let mut results = BTreeMap::new();
    let Some(extension) = extension.descriptor() else {
        return Ok(results);
    };

    let mut input = CelInput::Root;
    if field_options.has_extension(&extension) {
        let value = field_options.get_extension(&extension);
        let predef = value
            .as_message()
            .context("expected message")?
            .transcode_to::<tinc_pb::PredefinedConstraint>()
            .context("invalid predefined constraint")?;
        match predef.r#type() {
            tinc_pb::predefined_constraint::Type::Unspecified => {}
            tinc_pb::predefined_constraint::Type::CustomExpression => {}
            tinc_pb::predefined_constraint::Type::WrapperMapKey => {
                input = CelInput::MapKey;
            }
            tinc_pb::predefined_constraint::Type::WrapperMapValue => {
                input = CelInput::MapValue;
            }
            tinc_pb::predefined_constraint::Type::WrapperRepeatedItem => {
                input = CelInput::RepeatedItem;
            }
        }
    }

    for (ext, value) in field_options.extensions() {
        if &ext == extension {
            continue;
        }

        if let Some(message) = value.as_message() {
            explore_fields(extension, input, message, &mut results)?;
        }
    }

    Ok(results)
}

fn explore_fields(
    extension: &prost_reflect::ExtensionDescriptor,
    input: CelInput,
    value: &prost_reflect::DynamicMessage,
    results: &mut BTreeMap<CelInput, Vec<CelExpression>>,
) -> anyhow::Result<()> {
    for (field, value) in value.fields() {
        let options = field.options();
        let mut input = input;
        if options.has_extension(&extension) {
            let message = options.get_extension(&extension);
            let predef = message
                .as_message()
                .unwrap()
                .transcode_to::<tinc_pb::PredefinedConstraint>()
                .unwrap();
            match predef.r#type() {
                tinc_pb::predefined_constraint::Type::Unspecified => {}
                tinc_pb::predefined_constraint::Type::CustomExpression => {
                    if let Some(list) = value.as_list() {
                        let messages = list
                            .iter()
                            .filter_map(|item| item.as_message())
                            .filter_map(|msg| msg.transcode_to::<tinc_pb::CelExpression>().ok());
                        for message in messages {
                            let expr = CelExpression::new(&message, None)?;
                            results.entry(input).or_default().push(expr);
                        }
                    }
                    continue;
                }
                tinc_pb::predefined_constraint::Type::WrapperMapKey => {
                    input = CelInput::MapKey;
                }
                tinc_pb::predefined_constraint::Type::WrapperMapValue => {
                    input = CelInput::MapValue;
                }
                tinc_pb::predefined_constraint::Type::WrapperRepeatedItem => {
                    input = CelInput::RepeatedItem;
                }
            }

            for expr in &predef.cel {
                results
                    .entry(input)
                    .or_default()
                    .push(CelExpression::new(expr, Some(&value))?);
            }
        }

        let Some(message) = value.as_message() else {
            continue;
        };

        explore_fields(extension, input, message, results)?;
    }

    Ok(())
}
