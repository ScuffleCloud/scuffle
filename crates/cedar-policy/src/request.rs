use std::marker::PhantomData;

use crate::{CedarAction, CedarEntity, Entity};

/// A request builder used to construct [Request] types.
pub struct RequestBuilder<'a, A> {
    entities: Vec<cedar_policy::Entity>,
    schema: Option<&'a cedar_policy::Schema>,
    _marker: PhantomData<A>,
}

impl<A> Default for RequestBuilder<'_, A> {
    fn default() -> Self {
        Self::new(None)
    }
}

/// All the errors that can take place when building a request
#[derive(thiserror::Error, Debug)]
pub enum RequestBuilderError {
    /// Error while adding an entity
    #[error(transparent)]
    EntitiesError(#[from] Box<cedar_policy::entities_errors::EntitiesError>),
    /// Error while serializing to json
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Error while serializing the context
    #[error(transparent)]
    ContextJsonError(#[from] Box<cedar_policy::ContextJsonError>),
    /// Error while validating the request
    #[error(transparent)]
    RequestValidationError(#[from] Box<cedar_policy::RequestValidationError>),
}

impl<'a, A> RequestBuilder<'a, A> {
    /// Create a new request builder given a schema.
    pub fn new(schema: Option<&'a cedar_policy::Schema>) -> Self {
        Self {
            schema,
            entities: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Add an entity to the builder.
    /// See [Self::add_entity] for a mut ref builder method.
    pub fn with_entity<E: CedarEntity>(mut self, entity: &Entity<E>) -> Result<Self, RequestBuilderError> {
        self.add_entity(entity)?;
        Ok(self)
    }

    /// Add an entity to the builder.
    /// See [Self::with_entity] for a owned builder method.
    pub fn add_entity<E: CedarEntity>(&mut self, entity: &Entity<E>) -> Result<&mut Self, RequestBuilderError> {
        let entity = serde_json::to_value(entity)?;
        self.entities
            .push(cedar_policy::Entity::from_json_value(entity, self.schema).map_err(Box::new)?);
        Ok(self)
    }

    /// Build a request given a principal, resource and a ctx.
    pub fn build<P, R>(
        mut self,
        principal: &Entity<P>,
        resource: &Entity<R>,
        ctx: &A::Context,
    ) -> Result<Request, RequestBuilderError>
    where
        A: CedarAction<P, R>,
        P: CedarEntity,
        R: CedarEntity,
    {
        self.add_entity(principal)?;
        self.add_entity(resource)?;

        let entities = cedar_policy::Entities::empty()
            .add_entities(self.entities, self.schema)
            .map_err(Box::new)?;

        let action_euid = A::action_entity_uid();

        let context =
            cedar_policy::Context::from_json_value(serde_json::to_value(ctx)?, self.schema.map(|s| (s, action_euid)))
                .map_err(Box::new)?;

        let request = cedar_policy::Request::new(
            principal.entity_uid().into(),
            A::action_entity_uid().clone(),
            resource.entity_uid().into(),
            context,
            self.schema,
        )
        .map_err(Box::new)?;

        Ok(Request { entities, request })
    }
}

/// A request that can be used to authenticate
pub struct Request {
    entities: cedar_policy::Entities,
    request: cedar_policy::Request,
}

impl Request {
    /// Create a builder from a given schema.
    pub fn builder<'a, A>(schema: Option<&'a cedar_policy::Schema>) -> RequestBuilder<'a, A> {
        RequestBuilder::new(schema)
    }

    /// Check if the request is authorized.
    pub fn is_authorized(&self, policies: &cedar_policy::PolicySet) -> cedar_policy::Response {
        cedar_policy::Authorizer::new().is_authorized(&self.request, policies, &self.entities)
    }
}
