use cedar_policy_core::ast::Id;
use cedar_policy_core::validator::{CedarSchemaError, SchemaError};

/// Code generation errors
#[derive(thiserror::Error, Debug)]
pub enum CodegenError {
    /// Multiple types map to the same Id in the schema.
    #[error("multiple types with name {0}")]
    DuplicateType(Id),
    /// Something is unsupported.
    #[error("unsupported: {0}")]
    Unsupported(String),
    /// Multiple actions have the same name.
    #[error("multiple actions with name {0}")]
    DuplicateAction(String),
    /// A reference was not found.
    #[error("unresolved reference: {0}")]
    UnresolvedReference(String),
    /// Expected an entity but found a common type instead.
    #[error("expected entity not common type of {common_type} when declaring {ty}")]
    ExpectedEntity {
        /// The common type's name.
        common_type: String,
        /// The object we were declaring.
        ty: String,
    },
    /// Failed to parse cedar schema
    #[error(transparent)]
    CedarSchemaError(#[from] Box<CedarSchemaError>),
    /// Failed to parse json schema
    #[error(transparent)]
    SchemaError(#[from] Box<SchemaError>),
}

impl From<SchemaError> for CodegenError {
    fn from(value: SchemaError) -> Self {
        Self::SchemaError(Box::new(value))
    }
}

impl From<CedarSchemaError> for CodegenError {
    fn from(value: CedarSchemaError) -> Self {
        Self::CedarSchemaError(Box::new(value))
    }
}

/// A [Result] type with [CodegenError] as the default error.
pub type CodegenResult<T, E = CodegenError> = std::result::Result<T, E>;
