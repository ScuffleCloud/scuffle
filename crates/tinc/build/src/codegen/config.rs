use std::collections::BTreeMap;

use crate::types::ProtoPath;

#[derive(Default)]
pub(crate) struct AttributeConfig {
    enum_configs: BTreeMap<ProtoPath, EnumConfig>,
    message_configs: BTreeMap<ProtoPath, MessageConfig>,
}

impl AttributeConfig {
    pub(crate) fn enum_configs(&self) -> impl Iterator<Item = (&ProtoPath, &EnumConfig)> {
        self.enum_configs.iter()
    }

    pub(crate) fn message_configs(&self) -> impl Iterator<Item = (&ProtoPath, &MessageConfig)> {
        self.message_configs.iter()
    }

    pub(crate) fn enum_config(&mut self, name: &ProtoPath) -> &mut EnumConfig {
        self.enum_configs.entry(name.clone()).or_default()
    }

    pub(crate) fn message_config(&mut self, name: &ProtoPath) -> &mut MessageConfig {
        self.message_configs.entry(name.clone()).or_default()
    }
}

#[derive(Default)]
pub(crate) struct EnumConfig {
    container_attributes: Vec<syn::Attribute>,
    variant_attributes: BTreeMap<String, Vec<syn::Attribute>>,
}

impl EnumConfig {
    pub(crate) fn attributes(&self) -> impl Iterator<Item = &syn::Attribute> {
        self.container_attributes.iter()
    }

    pub(crate) fn variant_attributes(&self, variant: &str) -> impl Iterator<Item = &syn::Attribute> {
        self.variant_attributes.get(variant).into_iter().flatten()
    }

    pub(crate) fn variants(&self) -> impl Iterator<Item = &str> {
        self.variant_attributes.keys().map(String::as_str)
    }

    pub(crate) fn attribute(&mut self, attr: syn::Attribute) {
        self.container_attributes.push(attr);
    }

    pub(crate) fn variant_attribute(&mut self, variant: &str, attr: syn::Attribute) {
        self.variant_attributes.entry(variant.to_owned()).or_default().push(attr);
    }
}

#[derive(Default)]
pub(crate) struct MessageConfig {
    pub container_attributes: Vec<syn::Attribute>,
    pub field_attributes: BTreeMap<String, Vec<syn::Attribute>>,
    pub oneof_attributes: BTreeMap<String, OneofConfig>,
}

impl MessageConfig {
    pub(crate) fn attributes(&self) -> impl Iterator<Item = &syn::Attribute> {
        self.container_attributes.iter()
    }

    pub(crate) fn field_attributes(&self, field: &str) -> impl Iterator<Item = &syn::Attribute> {
        self.field_attributes.get(field).into_iter().flatten()
    }

    pub(crate) fn fields(&self) -> impl Iterator<Item = &str> {
        self.field_attributes.keys().map(String::as_str)
    }

    pub(crate) fn oneof_configs(&self) -> impl Iterator<Item = (&str, &OneofConfig)> {
        self.oneof_attributes.iter().map(|(name, config)| (name.as_str(), config))
    }

    pub(crate) fn attribute(&mut self, attr: syn::Attribute) {
        self.container_attributes.push(attr);
    }

    pub(crate) fn field_attribute(&mut self, field: &str, attr: syn::Attribute) {
        self.field_attributes.entry(field.to_owned()).or_default().push(attr);
    }

    pub(crate) fn oneof_config(&mut self, oneof: &str) -> &mut OneofConfig {
        self.oneof_attributes.entry(oneof.to_owned()).or_default()
    }
}

#[derive(Default)]
pub(crate) struct OneofConfig {
    pub container_attributes: Vec<syn::Attribute>,
    pub field_attributes: BTreeMap<String, Vec<syn::Attribute>>,
}

impl OneofConfig {
    pub(crate) fn attributes(&self) -> impl Iterator<Item = &syn::Attribute> {
        self.container_attributes.iter()
    }

    pub(crate) fn field_attributes(&self, field: &str) -> impl Iterator<Item = &syn::Attribute> {
        self.field_attributes.get(field).into_iter().flatten()
    }

    pub(crate) fn fields(&self) -> impl Iterator<Item = &str> {
        self.field_attributes.keys().map(String::as_str)
    }

    pub(crate) fn attribute(&mut self, attr: syn::Attribute) {
        self.container_attributes.push(attr);
    }

    pub(crate) fn field_attribute(&mut self, field: &str, attr: syn::Attribute) {
        self.field_attributes.entry(field.to_owned()).or_default().push(attr);
    }
}
