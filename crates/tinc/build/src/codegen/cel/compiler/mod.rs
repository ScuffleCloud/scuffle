use std::collections::BTreeMap;

use quote::ToTokens;
use syn::parse_quote;

use super::FuncFmtter;
use super::functions::Function;
use super::types::CelType;
use crate::types::{ProtoPath, ProtoTypeRegistry};

mod helpers;
mod resolve;

#[derive(Clone)]
pub struct CompiledExpr {
    pub expr: syn::Expr,
    pub ty: CelType,
}

impl std::fmt::Debug for CompiledExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledExpr")
            .field("ty", &self.ty)
            .field(
                "expr",
                &FuncFmtter(|fmt| {
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

impl ToTokens for CompiledExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.expr.to_tokens(tokens);
    }
}

#[derive(Clone, Debug)]
pub struct Compiler<'a> {
    parent: Option<&'a Compiler<'a>>,
    registry: &'a ProtoTypeRegistry,
    variables: BTreeMap<String, CompiledExpr>,
    functions: BTreeMap<&'static str, CompilerFunction>,
}

impl<'a> Compiler<'a> {
    pub fn empty(registry: &'a ProtoTypeRegistry) -> Self {
        Self {
            parent: None,
            registry,
            variables: BTreeMap::new(),
            functions: BTreeMap::new(),
        }
    }

    fn child(&self) -> Compiler<'_> {
        Compiler {
            parent: Some(self),
            registry: self.registry,
            variables: BTreeMap::new(),
            functions: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompilerCtx<'a> {
    pub this: Option<CompiledExpr>,
    pub args: &'a [cel_parser::Expression],
    compiler: Compiler<'a>,
}

impl<'a> CompilerCtx<'a> {
    pub fn new(compiler: Compiler<'a>, this: Option<CompiledExpr>, args: &'a [cel_parser::Expression]) -> Self {
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
    pub fn add_variable(&mut self, name: &str, expr: CompiledExpr) {
        self.variables.insert(name.to_owned(), expr.clone());
    }

    pub fn register_function<F: Function>(&mut self) {
        if self
            .functions
            .insert(F::NAME, CompilerFunction { compile: F::compile })
            .is_some()
        {
            panic!("function {} already registered", F::NAME);
        }
    }

    pub fn resolve(&self, expr: &cel_parser::Expression) -> Result<CompiledExpr, CompileError> {
        resolve::resolve(self, expr)
    }

    pub fn get_variable(&self, name: &str) -> Option<&CompiledExpr> {
        match self.variables.get(name) {
            Some(expr) => Some(expr),
            None => match self.parent {
                Some(parent) => parent.get_variable(name),
                None => None,
            },
        }
    }

    pub fn get_function(&self, name: &str) -> Option<&CompilerFunction> {
        match self.functions.get(name) {
            Some(func) => Some(func),
            None => match self.parent {
                Some(parent) => parent.get_function(name),
                None => None,
            },
        }
    }

    pub fn registry(&self) -> &'a ProtoTypeRegistry {
        self.registry
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CompileError {
    #[error("not implemented")]
    NotImplemented,
    #[error("missing target for function {func}: {message}")]
    MissingTarget { func: &'static str, message: String },
    #[error("invalid function argument count for {func}, expected {expected} got {got}")]
    InvalidFunctionArgumentCount {
        func: &'static str,
        expected: usize,
        got: usize,
    },
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
    pub fn type_conversion(ty: CelType, message: impl Into<String>) -> Self {
        Self::TypeConversion {
            ty: Box::new(ty),
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct CompilerFunction {
    pub compile: fn(ctx: CompilerCtx) -> Result<CompiledExpr, CompileError>,
}

impl std::fmt::Debug for CompilerFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CompilerFunction")
    }
}
