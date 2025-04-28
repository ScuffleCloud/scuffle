use std::collections::BTreeMap;

use service::handle_service;
use types::{ProtoPath, ProtoTypeRegistry};

use self::serde::{handle_enum, handle_message};

pub mod cel;
mod explore;
mod prost_sanatize;
mod serde;
mod service;
pub mod types;
mod utils;

pub use explore::Extensions;

pub fn generate_modules(
    registry: &ProtoTypeRegistry,
    prost: &mut tonic_build::Config,
) -> anyhow::Result<BTreeMap<ProtoPath, Vec<syn::Item>>> {
    let mut modules = BTreeMap::new();

    registry
        .messages()
        .try_for_each(|message| handle_message(message, prost, &mut modules, registry))?;

    registry
        .enums()
        .try_for_each(|enum_| handle_enum(enum_, prost, &mut modules))?;

    registry
        .services()
        .try_for_each(|service| handle_service(service, &mut modules, registry))?;

    Ok(modules)
}
