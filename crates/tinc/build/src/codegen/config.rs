use std::collections::BTreeMap;

use crate::types::ProtoPath;

#[derive(Default, Debug)]
pub struct AttributeConfig {
    enum_configs: BTreeMap<ProtoPath, EnumConfig>,
    message_configs: BTreeMap<ProtoPath, MessageConfig>,
}

impl AttributeConfig {
    pub fn enum_configs(&self) -> impl Iterator<Item = (&ProtoPath, &EnumConfig)> {
        self.enum_configs.iter()
    }

    pub fn message_configs(&self) -> impl Iterator<Item = (&ProtoPath, &MessageConfig)> {
        self.message_configs.iter()
    }

    pub fn enum_config(&mut self, name: &ProtoPath) -> &mut EnumConfig {
        self.enum_configs.entry(name.clone()).or_default()
    }

    pub fn message_config(&mut self, name: &ProtoPath) -> &mut MessageConfig {
        self.message_configs.entry(name.clone()).or_default()
    }
}

#[derive(Default, Debug)]
pub struct EnumConfig {
    container_attributes: Vec<syn::Attribute>,
    variant_attributes: BTreeMap<String, Vec<syn::Attribute>>,
}

impl EnumConfig {
    pub fn attributes(&self) -> impl Iterator<Item = &syn::Attribute> {
        self.container_attributes.iter()
    }

    pub fn variant_attributes(&self, variant: &str) -> impl Iterator<Item = &syn::Attribute> {
        self.variant_attributes.get(variant).into_iter().flatten()
    }

    pub fn variants(&self) -> impl Iterator<Item = &str> {
        self.variant_attributes.keys().map(String::as_str)
    }

    pub fn attribute(&mut self, attr: syn::Attribute) {
        self.container_attributes.push(attr);
    }

    pub fn variant_attribute(&mut self, variant: &str, attr: syn::Attribute) {
        self.variant_attributes.entry(variant.to_owned()).or_default().push(attr);
    }
}

#[derive(Default, Debug)]
pub struct MessageConfig {
    pub container_attributes: Vec<syn::Attribute>,
    pub field_attributes: BTreeMap<String, Vec<syn::Attribute>>,
    pub oneof_attributes: BTreeMap<String, OneofConfig>,
}

impl MessageConfig {
    pub fn attributes(&self) -> impl Iterator<Item = &syn::Attribute> {
        self.container_attributes.iter()
    }

    pub fn field_attributes(&self, field: &str) -> impl Iterator<Item = &syn::Attribute> {
        self.field_attributes.get(field).into_iter().flatten()
    }

    pub fn fields(&self) -> impl Iterator<Item = &str> {
        self.field_attributes.keys().map(String::as_str)
    }

    pub fn oneof_configs(&self) -> impl Iterator<Item = (&str, &OneofConfig)> {
        self.oneof_attributes.iter().map(|(name, config)| (name.as_str(), config))
    }

    pub fn attribute(&mut self, attr: syn::Attribute) {
        self.container_attributes.push(attr);
    }

    pub fn field_attribute(&mut self, field: &str, attr: syn::Attribute) {
        self.field_attributes.entry(field.to_owned()).or_default().push(attr);
    }

    pub fn oneof_config(&mut self, oneof: &str) -> &mut OneofConfig {
        self.oneof_attributes.entry(oneof.to_owned()).or_default()
    }
}

#[derive(Default, Debug)]
pub struct OneofConfig {
    pub container_attributes: Vec<syn::Attribute>,
    pub field_attributes: BTreeMap<String, Vec<syn::Attribute>>,
}

impl OneofConfig {
    pub fn attributes(&self) -> impl Iterator<Item = &syn::Attribute> {
        self.container_attributes.iter()
    }

    pub fn field_attributes(&self, field: &str) -> impl Iterator<Item = &syn::Attribute> {
        self.field_attributes.get(field).into_iter().flatten()
    }

    pub fn fields(&self) -> impl Iterator<Item = &str> {
        self.field_attributes.keys().map(String::as_str)
    }

    pub fn attribute(&mut self, attr: syn::Attribute) {
        self.container_attributes.push(attr);
    }

    pub fn field_attribute(&mut self, field: &str, attr: syn::Attribute) {
        self.field_attributes.entry(field.to_owned()).or_default().push(attr);
    }
}
