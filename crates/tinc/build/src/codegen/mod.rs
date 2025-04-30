use std::collections::BTreeMap;

pub use config::AttributeConfig;
use service::handle_service;

use self::serde::{handle_enum, handle_message};
use crate::types::{ProtoPath, ProtoService, ProtoTypeRegistry};

pub mod cel;
mod config;
pub mod prost_sanatize;
mod serde;
mod service;
pub mod utils;

#[derive(Default, Debug)]
pub struct Package {
    pub attributes: AttributeConfig,
    pub extra_items: Vec<syn::Item>,
    pub services: Vec<ProtoService>,
}

impl Package {
    pub fn push_item(&mut self, item: syn::Item) {
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

pub fn generate_modules(registry: &ProtoTypeRegistry) -> anyhow::Result<BTreeMap<ProtoPath, Package>> {
    let mut modules = BTreeMap::new();

    registry
        .messages()
        .try_for_each(|message| handle_message(message, modules.entry(message.package.clone()).or_default(), registry))?;

    registry
        .enums()
        .try_for_each(|enum_| handle_enum(enum_, modules.entry(enum_.package.clone()).or_default()))?;

    registry.services().try_for_each(|service| {
        modules
            .entry(service.package.clone())
            .or_default()
            .services
            .push(service.clone());
        handle_service(service, modules.entry(service.package.clone()).or_default(), registry)
    })?;

    Ok(modules)
}
