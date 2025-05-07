use std::collections::BTreeMap;
use std::sync::Arc;

use quote::{ToTokens, quote};
use syn::parse_quote;
use tinc_cel::{CelEnum, CelError, CelValue, CelValueConv};

use super::functions::{Function, add_to_compiler};
use super::types::CelType;
use crate::types::{ProtoPath, ProtoTypeRegistry};

mod helpers;
mod resolve;

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum CompiledExpr {
    Runtime(RuntimeCompiledExpr),
    Constant(ConstantCompiledExpr),
}

impl CompiledExpr {
    pub(crate) fn constant(value: impl CelValueConv<'static>) -> Self {
        Self::Constant(ConstantCompiledExpr { value: value.conv() })
    }

    pub(crate) fn runtime(ty: CelType, expr: syn::Expr) -> Self {
        Self::Runtime(RuntimeCompiledExpr { expr, ty })
    }
}

#[derive(Clone)]
pub(crate) struct RuntimeCompiledExpr {
    pub expr: syn::Expr,
    pub ty: CelType,
}

#[derive(Debug, Clone)]
pub(crate) struct ConstantCompiledExpr {
    pub value: tinc_cel::CelValue<'static>,
}

impl std::fmt::Debug for RuntimeCompiledExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeCompiledExpr")
            .field("ty", &self.ty)
            .field(
                "expr",
                &fmtools::fmt(|fmt| {
                    let expr = &self.expr;
                    let tokens = parse_quote! {
                        const _: Debug = #expr;
                    };
                    let pretty = prettyplease::unparse(&tokens);
                    let pretty = pretty.trim();
                    let pretty = pretty.strip_prefix("const _: Debug =").unwrap_or(pretty);
                    let pretty = pretty.strip_suffix(';').unwrap_or(pretty);
                    fmt.write_str(pretty.trim())
                }),
            )
            .finish()
    }
}

impl ToTokens for RuntimeCompiledExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.expr.to_tokens(tokens);
    }
}

impl ToTokens for ConstantCompiledExpr {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        fn value_to_tokens(value: &CelValue) -> proc_macro2::TokenStream {
            match value {
                CelValue::Bool(b) => quote! {
                    ::tinc::__private::cel::CelValue::Bool(#b)
                },
                CelValue::Bytes(b) => {
                    let b = syn::LitByteStr::new(b.as_ref(), proc_macro2::Span::call_site());
                    quote! {
                        ::tinc::__private::cel::CelValue::Bytes(
                            ::tinc::__private::cel::CelBytes::Borrowed(
                                #b
                            )
                        )
                    }
                }
                CelValue::Enum(CelEnum { tag, value }) => {
                    let tag = tag.as_ref();
                    quote! {
                        ::tinc::__private::cel::CelValue::Enum(
                            ::tinc::__private::cel::CelValue::CelEnum {
                                tag: ::tinc::__private::cel::CelValue::CelString::Borrowed(#tag),
                                value: #value,
                            }
                        )
                    }
                }
                CelValue::List(list) => {
                    let list = list.iter().map(value_to_tokens);
                    quote! {
                        ::tinc::__private::cel::CelValue::List([
                            #(#list),*
                        ].into_iter().collect())
                    }
                }
                CelValue::Map(map) => {
                    let map = map
                        .iter()
                        .map(|(key, value)| (value_to_tokens(key), value_to_tokens(value)))
                        .map(|(key, value)| quote!((#key, #value)));
                    quote! {
                        ::tinc::__private::cel::CelValue::List([
                            #(#map),*
                        ].into_iter().collect())
                    }
                }
                CelValue::Null => quote!(::tinc::__private::cel::CelValue::Null),
                CelValue::Number(tinc_cel::NumberTy::F64(f)) => {
                    quote!(::tinc::__private::cel::CelValue::Number(::tinc::__private::cel::NumberTy::F64(#f)))
                }
                CelValue::Number(tinc_cel::NumberTy::I64(i)) => {
                    quote!(::tinc::__private::cel::CelValue::Number(::tinc::__private::cel::NumberTy::I64(#i)))
                }
                CelValue::Number(tinc_cel::NumberTy::U64(u)) => {
                    quote!(::tinc::__private::cel::CelValue::Number(::tinc::__private::cel::NumberTy::U64(#u)))
                }
                CelValue::String(s) => {
                    let s = s.as_ref();
                    quote!(::tinc::__private::cel::CelValue::String(::tinc::__private::cel::CelString::Borrowed(#s)))
                }
                CelValue::Duration(b) => {
                    let secs = b.num_seconds();
                    let nanos = b.subsec_nanos();
                    quote! {
                        ::tinc::__private::cel::CelValue::Duration(
                            ::tinc::reexports::chrono::Duration::new(
                                #secs,
                                #nanos,
                            ).expect("duration was valid at build")
                        )
                    }
                }
                CelValue::Timestamp(ts) => {
                    let tz_offset = ts.offset().local_minus_utc();
                    let utc = ts.to_utc();
                    let ts_secs = utc.timestamp();
                    let ts_nanos = utc.timestamp_subsec_nanos();
                    quote! {
                        ::tinc::__private::cel::CelValue::Timestamp(
                            ::tinc::reexports::chrono::TimeZone::timestamp_opt(
                                &::tinc::reexports::chrono::offset::FixedOffset::east_opt(#tz_offset)
                                    .expect("codegen from build"),
                                #ts_secs,
                                #ts_nanos,
                            ).unwrap()
                        )
                    }
                }
            }
        }

        value_to_tokens(&self.value).to_tokens(stream);
    }
}

impl ToTokens for CompiledExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Constant(c) => c.to_tokens(tokens),
            Self::Runtime(r) => r.to_tokens(tokens),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompilerTarget {
    Serde,
    Proto,
}

#[derive(Clone, Debug)]
pub(crate) struct Compiler<'a> {
    parent: Option<&'a Compiler<'a>>,
    registry: &'a ProtoTypeRegistry,
    target: Option<CompilerTarget>,
    variables: BTreeMap<String, CompiledExpr>,
    functions: BTreeMap<&'static str, DebugFunc>,
}

#[derive(Clone)]
struct DebugFunc(Arc<dyn Function + Send + Sync + 'static>);
impl std::fmt::Debug for DebugFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.name())
    }
}

impl<'a> Compiler<'a> {
    pub(crate) fn empty(registry: &'a ProtoTypeRegistry) -> Self {
        Self {
            parent: None,
            registry,
            target: None,
            variables: BTreeMap::new(),
            functions: BTreeMap::new(),
        }
    }

    pub(crate) fn new(registry: &'a ProtoTypeRegistry) -> Self {
        let mut compiler = Self::empty(registry);

        add_to_compiler(&mut compiler);

        compiler
    }

    #[allow(dead_code)]
    pub(crate) fn set_target(&mut self, target: impl Into<Option<CompilerTarget>>) {
        self.target = target.into()
    }

    pub(crate) fn target(&self) -> Option<CompilerTarget> {
        self.target
    }

    pub(crate) fn child(&self) -> Compiler<'_> {
        Compiler {
            parent: Some(self),
            registry: self.registry,
            target: self.target,
            variables: BTreeMap::new(),
            functions: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CompilerCtx<'a> {
    pub this: Option<CompiledExpr>,
    pub args: &'a [cel_parser::Expression],
    compiler: Compiler<'a>,
}

impl<'a> CompilerCtx<'a> {
    pub(crate) fn new(compiler: Compiler<'a>, this: Option<CompiledExpr>, args: &'a [cel_parser::Expression]) -> Self {
        Self { this, args, compiler }
    }
}

impl<'a> std::ops::Deref for CompilerCtx<'a> {
    type Target = Compiler<'a>;

    fn deref(&self) -> &Self::Target {
        &self.compiler
    }
}

impl std::ops::DerefMut for CompilerCtx<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.compiler
    }
}

impl<'a> Compiler<'a> {
    pub(crate) fn add_variable(&mut self, name: &str, expr: CompiledExpr) {
        self.variables.insert(name.to_owned(), expr.clone());
    }

    pub(crate) fn register_function(&mut self, f: impl Function) {
        let name = f.name();
        self.functions.insert(name, DebugFunc(Arc::new(f)));
    }

    pub(crate) fn resolve(&self, expr: &cel_parser::Expression) -> Result<CompiledExpr, CompileError> {
        resolve::resolve(self, expr)
    }

    pub(crate) fn get_variable(&self, name: &str) -> Option<&CompiledExpr> {
        match self.variables.get(name) {
            Some(expr) => Some(expr),
            None => match self.parent {
                Some(parent) => parent.get_variable(name),
                None => None,
            },
        }
    }

    pub(crate) fn get_function(&self, name: &str) -> Option<&Arc<dyn Function + Send + Sync + 'static>> {
        match self.functions.get(name) {
            Some(func) => Some(&func.0),
            None => match self.parent {
                Some(parent) => parent.get_function(name),
                None => None,
            },
        }
    }

    pub(crate) fn registry(&self) -> &'a ProtoTypeRegistry {
        self.registry
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub(crate) enum CompileError {
    #[error("not implemented")]
    NotImplemented,
    #[error("invalid syntax: {message} - {syntax}")]
    InvalidSyntax { message: String, syntax: &'static str },
    #[error("type conversion error on type {ty:?}: {message}")]
    TypeConversion { ty: Box<CelType>, message: String },
    #[error("member access error on type {ty:?}: {message}")]
    MemberAccess { ty: Box<CelType>, message: String },
    #[error("variable not found: {0}")]
    VariableNotFound(String),
    #[error("function not found: {0}")]
    FunctionNotFound(String),
    #[error("unsupported function call identifier type: {0:?}")]
    UnsupportedFunctionCallIdentifierType(cel_parser::Expression),
    #[error("missing message: {0}")]
    MissingMessage(ProtoPath),
}

impl CompileError {
    pub(crate) fn syntax(message: impl std::fmt::Display, func: &impl Function) -> CompileError {
        CompileError::InvalidSyntax {
            message: message.to_string(),
            syntax: func.syntax(),
        }
    }
}

impl From<CelError<'_>> for CompileError {
    fn from(value: CelError<'_>) -> Self {
        Self::TypeConversion {
            ty: Box::new(CelType::CelValue),
            message: value.to_string(),
        }
    }
}
