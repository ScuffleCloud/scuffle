use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use cedar_policy_core::ast::Id;
use cedar_policy_core::validator::RawName;

use crate::utils::{to_snake_ident, to_upper_camel_ident};

/// Represents a reference to a Cedar type with its namespace
#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub(crate) struct CedarRef {
    pub(crate) id: Id,
    pub(crate) namespace: NamespaceId,
}

impl From<RawName> for CedarRef {
    fn from(value: RawName) -> Self {
        let qualified_value = value.qualify_with(None);
        Self {
            id: qualified_value.basename().clone(),
            namespace: NamespaceId {
                items: qualified_value.namespace_components().cloned().collect(),
            },
        }
    }
}

impl CedarRef {
    /// Converts this reference to a CedarType
    pub(crate) fn into_cedar_ty(self) -> CedarType {
        match self.id.as_ref() {
            "Bool" => CedarType::Bool,
            "Long" => CedarType::Long,
            "String" => CedarType::String,
            _ => CedarType::Reference(self),
        }
    }

    /// Gets the identifier path for this reference
    pub(crate) fn ident_path(&self) -> Vec<syn::Ident> {
        self.namespace
            .items
            .iter()
            .map(to_snake_ident)
            .chain(std::iter::once(&self.id).map(to_upper_camel_ident))
            .collect()
    }
}

/// Represents a namespace identifier with components
#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Default, Clone)]
pub(crate) struct NamespaceId {
    pub items: Vec<Id>,
}

/// Represents a field within a Cedar record type
#[derive(Debug)]
pub(crate) struct CedarTypeStructField {
    pub ty: CedarType,
    pub optional: bool,
}

/// Represents an action entity identifier
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub(crate) struct ActionEid {
    pub id: Option<CedarRef>,
    pub name: String,
}

/// Represents different Cedar types
#[derive(Debug)]
pub(crate) enum CedarType {
    String,
    Long,
    Bool,
    Record {
        fields: BTreeMap<String, CedarTypeStructField>,
        allows_additional: bool,
    },
    Set(Box<CedarType>),
    Enum(BTreeSet<String>),
    Reference(CedarRef),
    Entity {
        parents: Vec<CedarRef>,
        shape: Box<CedarType>,
        tag_type: Option<Box<CedarType>>,
    },
}

impl CedarType {
    /// Returns true if this type represents an entity
    pub(crate) fn is_entity(&self) -> bool {
        matches!(self, Self::Entity { .. } | Self::Enum(_))
    }
}

impl std::fmt::Display for ActionEid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = &self.id {
            id.fmt(f)?;
            f.write_str("::")?;
        }

        f.write_char('"')?;
        f.write_str(&self.name)?;
        f.write_char('"')
    }
}

impl std::fmt::Debug for NamespaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NamespaceId(\"{self}\")")
    }
}

impl std::fmt::Display for CedarRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.namespace.fmt(f)?;
        if !self.namespace.items.is_empty() {
            f.write_str("::")?;
        }
        self.id.fmt(f)
    }
}

impl std::fmt::Display for NamespaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for part in &self.items {
            if !first {
                f.write_str("::")?;
            }
            part.fmt(f)?;
            first = false;
        }
        Ok(())
    }
}
