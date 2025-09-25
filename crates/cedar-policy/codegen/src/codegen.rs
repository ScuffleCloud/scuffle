use std::collections::BTreeMap;

use cedar_policy_core::ast::Id;

use crate::Config;
use crate::cedar_action::CedarAction;
use crate::cedar_namespace::CedarNamespace;
use crate::error::CodegenError;
use crate::module::Module;
use crate::types::{ActionEid, CedarRef, CedarType, NamespaceId};

/// Main code generation structure
pub(crate) struct Codegen<'a> {
    config: &'a Config,
    namespaces: BTreeMap<NamespaceId, CedarNamespace>,
}

impl<'a> Codegen<'a> {
    pub(crate) fn new(config: &'a Config) -> Self {
        Self {
            config,
            namespaces: BTreeMap::default(),
        }
    }

    pub(crate) fn config(&self) -> &'a Config {
        self.config
    }

    pub(crate) fn add_namespace(&mut self, id: NamespaceId, ns: CedarNamespace) {
        self.namespaces.insert(id, ns);
    }

    pub(crate) fn generate(&self) -> Result<syn::File, CodegenError> {
        let mut root = Module::default();

        for (ns_id, ns) in &self.namespaces {
            let module = self.get_namespace_module(&mut root, ns_id);
            self.generate_types(ns_id, module, &ns.types)?;
            self.generate_actions(ns_id, module, &ns.actions)?;
        }

        Ok(syn::File {
            attrs: Vec::new(),
            items: root.into_items(),
            shebang: None,
        })
    }

    fn get_namespace_module<'b>(&self, root: &'b mut Module, ns_id: &NamespaceId) -> &'b mut Module {
        ns_id.items.iter().fold(root, |module, id| module.sub_module(id))
    }

    fn generate_types(
        &self,
        ns_id: &NamespaceId,
        module: &mut Module,
        types: &BTreeMap<Id, CedarType>,
    ) -> Result<(), CodegenError> {
        for (id, ty) in types {
            module.handle_type(self, ns_id, id, ty)?;
        }
        Ok(())
    }

    fn generate_actions(
        &self,
        ns_id: &NamespaceId,
        module: &mut Module,
        actions: &BTreeMap<String, CedarAction>,
    ) -> Result<(), CodegenError> {
        for (action, ty) in actions {
            module.sub_module("action").handle_action(self, ns_id, action, ty)?;
        }
        Ok(())
    }

    pub(crate) fn resolve_ref(&self, reference: &CedarRef) -> Option<&CedarType> {
        self.namespaces.get(&reference.namespace)?.types.get(&reference.id)
    }

    pub(crate) fn contains_action(&self, ns: &NamespaceId, action: &ActionEid) -> bool {
        if action.id.as_ref().is_some_and(|id| id.id.as_ref() != "Action") {
            return false;
        }

        self.namespaces
            .get(action.id.as_ref().map(|i| &i.namespace).unwrap_or(ns))
            .is_some_and(|ns| ns.actions.contains_key(&action.name))
    }
}
