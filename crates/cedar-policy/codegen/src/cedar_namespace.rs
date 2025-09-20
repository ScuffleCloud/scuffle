use std::collections::BTreeMap;

use cedar_policy_core::ast::{Id, UnreservedId};
use cedar_policy_core::validator::RawName;
use cedar_policy_core::validator::json_schema::{self, ActionType, CommonType, CommonTypeId, EntityType};

use crate::cedar_action::CedarAction;
use crate::error::CodegenError;
use crate::types::{ActionEid, CedarType};
use crate::utils::convert_cedar_to_rust;

/// Represents a Cedar namespace containing types and actions
#[derive(Default, Debug)]
pub(crate) struct CedarNamespace {
    pub types: BTreeMap<Id, CedarType>,
    pub actions: BTreeMap<String, CedarAction>,
}

impl CedarNamespace {
    /// Handles a common type definition
    pub(crate) fn handle_common_type(&mut self, id: &CommonTypeId, ty: &CommonType<RawName>) -> Result<(), CodegenError> {
        let id = id.as_ref().clone().into();
        if self.types.contains_key(&id) {
            return Err(CodegenError::DuplicateType(id));
        }

        self.types.insert(id, convert_cedar_to_rust(&ty.ty)?);
        Ok(())
    }

    /// Handles an entity type definition
    pub(crate) fn handle_entity_type(&mut self, id: &UnreservedId, ty: &EntityType<RawName>) -> Result<(), CodegenError> {
        let id = id.clone().into();
        if self.types.contains_key(&id) {
            return Err(CodegenError::DuplicateType(id));
        }

        let cedar_type = match &ty.kind {
            json_schema::EntityTypeKind::Enum { choices } => {
                CedarType::Enum(choices.iter().map(|c| c.to_string()).collect())
            }
            json_schema::EntityTypeKind::Standard(std) => CedarType::Entity {
                parents: std.member_of_types.iter().cloned().map(Into::into).collect(),
                tag_type: std.tags.as_ref().map(convert_cedar_to_rust).transpose()?.map(Box::new),
                shape: Box::new(convert_cedar_to_rust(&std.shape.0)?),
            },
        };

        self.types.insert(id, cedar_type);
        Ok(())
    }

    /// Handles an action definition
    pub(crate) fn handle_action(&mut self, action: &str, ty: &ActionType<RawName>) -> Result<(), CodegenError> {
        if self.actions.contains_key(action) {
            return Err(CodegenError::DuplicateAction(action.to_string()));
        }

        let member_of = ty
            .member_of
            .as_ref()
            .map(|m| m.iter())
            .into_iter()
            .flatten()
            .map(|m| ActionEid {
                id: m.ty.clone().map(Into::into),
                name: m.id.to_string(),
            })
            .collect();

        let mut cedar_action = CedarAction {
            parents: member_of,
            ..Default::default()
        };

        if let Some(applies_to) = &ty.applies_to {
            cedar_action.context = self.extract_context_type(&applies_to.context.0)?;
            cedar_action.principals = applies_to.principal_types.iter().cloned().map(Into::into).collect();
            cedar_action.resources = applies_to.resource_types.iter().cloned().map(Into::into).collect();
        }

        self.actions.insert(action.to_owned(), cedar_action);
        Ok(())
    }

    /// Extracts context type from JSON schema type
    fn extract_context_type(&self, context_type: &json_schema::Type<RawName>) -> Result<Option<CedarType>, CodegenError> {
        match context_type {
            json_schema::Type::Type {
                ty: json_schema::TypeVariant::Record(r),
                ..
            } if !r.additional_attributes && r.attributes.is_empty() => Ok(None),
            r => Ok(Some(convert_cedar_to_rust(r)?)),
        }
    }
}
