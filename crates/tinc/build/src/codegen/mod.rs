use std::collections::BTreeMap;

pub(crate) use config::AttributeConfig;
pub(crate) use service::SchemaRegistry;
use service::{ProcessedService, handle_service};

use self::serde::{handle_enum, handle_message};
use crate::types::{ProtoPath, ProtoTypeRegistry};

pub(crate) mod cel;
mod config;
pub(crate) mod prost_sanatize;
mod serde;
mod service;
pub(crate) mod utils;

#[derive(Default)]
pub(crate) struct Package {
    pub attributes: AttributeConfig,
    pub extra_items: Vec<syn::Item>,
    pub services: Vec<ProcessedService>,
}

impl Package {
    pub(crate) fn push_item(&mut self, item: syn::Item) {
        self.extra_items.push(item);
    }
}

impl std::ops::Deref for Package {
    type Target = AttributeConfig;

    fn deref(&self) -> &Self::Target {
        &self.attributes
    }
}

impl std::ops::DerefMut for Package {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.attributes
    }
}

pub(crate) fn generate_modules(
    registry: &ProtoTypeRegistry,
    schema: &mut SchemaRegistry,
) -> anyhow::Result<BTreeMap<ProtoPath, Package>> {
    let mut modules = BTreeMap::new();

    registry
        .messages()
        .filter(|message| !registry.has_extern(&message.full_name))
        .try_for_each(|message| handle_message(message, modules.entry(message.package.clone()).or_default(), registry))?;

    registry
        .enums()
        .filter(|enum_| !registry.has_extern(&enum_.full_name))
        .try_for_each(|enum_| handle_enum(enum_, modules.entry(enum_.package.clone()).or_default(), registry))?;

    registry.services().try_for_each(|service| {
        handle_service(service, modules.entry(service.package.clone()).or_default(), registry, schema)
    })?;

    Ok(modules)
}
