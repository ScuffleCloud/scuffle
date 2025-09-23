use crate::{CedarEntity, Entity};

/// A request builder used to construct [Request] types.
pub struct EntitiesBuilder<'a> {
    entities: Vec<cedar_policy::Entity>,
    schema: Option<&'a cedar_policy::Schema>,
}

impl Default for EntitiesBuilder<'_> {
    fn default() -> Self {
        Self::new(None)
    }
}

/// All the errors that can take place when building a request
#[derive(thiserror::Error, Debug)]
pub enum EntitiesBuilderError {
    /// Error while adding an entity
    #[error(transparent)]
    EntitiesError(#[from] Box<cedar_policy::entities_errors::EntitiesError>),
    /// Error while serializing to json
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl<'a> EntitiesBuilder<'a> {
    /// Create a new request builder given a schema.
    pub fn new(schema: Option<&'a cedar_policy::Schema>) -> Self {
        Self {
            schema,
            entities: Vec::new(),
        }
    }

    /// Add an entity to the builder.
    /// See [Self::add_entity] for a mut ref builder method.
    pub fn with_entity<E: CedarEntity>(mut self, entity: &Entity<E>) -> Result<Self, EntitiesBuilderError> {
        self.add_entity(entity)?;
        Ok(self)
    }

    /// Add an entity to the builder.
    /// See [Self::with_entity] for a owned builder method.
    pub fn add_entity<E: CedarEntity>(&mut self, entity: &Entity<E>) -> Result<&mut Self, EntitiesBuilderError> {
        let entity = serde_json::to_value(entity)?;
        self.entities
            .push(cedar_policy::Entity::from_json_value(entity, self.schema).map_err(Box::new)?);
        Ok(self)
    }

    /// Build a request given a principal, resource and a ctx.
    pub fn build(self) -> Result<cedar_policy::Entities, EntitiesBuilderError> {
        cedar_policy::Entities::empty()
            .add_entities(self.entities, self.schema)
            .map_err(Box::new)
            .map_err(EntitiesBuilderError::EntitiesError)
    }
}
