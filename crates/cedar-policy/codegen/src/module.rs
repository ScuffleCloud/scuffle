use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use cedar_policy_core::ast::Id;

use crate::cedar_action::CedarAction;
use crate::codegen::Codegen;
use crate::error::{CodegenError, CodegenResult};
use crate::types::{CedarRef, CedarType, CedarTypeStructField, NamespaceId};
use crate::utils::{find_relative_path, to_snake_ident, to_upper_camel_ident};

/// Represents a module with its path and items
#[derive(Default)]
pub(crate) struct Module {
    root_path: Vec<syn::Ident>,
    items: Vec<syn::Item>,
    sub_modules: BTreeMap<syn::Ident, Module>,
}

impl Module {
    /// Gets or creates a sub-module
    pub(crate) fn sub_module(&mut self, name: impl AsRef<str>) -> &mut Self {
        let ident = to_snake_ident(name);
        self.sub_modules.entry(ident.clone()).or_insert_with(|| Module {
            root_path: {
                let mut path = self.root_path.clone();
                path.push(ident);
                path
            },
            items: Vec::new(),
            sub_modules: BTreeMap::new(),
        })
    }

    /// Converts this module into syntax items
    pub(crate) fn into_items(self) -> Vec<syn::Item> {
        let mut items = self.items;

        for (ident, module) in self.sub_modules {
            let mod_items = module.into_items();
            if !mod_items.is_empty() {
                items.push(syn::parse_quote! {
                    pub mod #ident {
                        #(#mod_items)*
                    }
                });
            }
        }

        items
    }

    /// Handles sub-type generation
    fn handle_sub_type(
        &mut self,
        codegen: &Codegen,
        ns: &NamespaceId,
        in_submodule: bool,
        name: impl AsRef<str>,
        ty: &CedarType,
    ) -> CodegenResult<syn::Type> {
        let name = name.as_ref();
        match ty {
            CedarType::Bool => Ok(syn::parse_quote!(bool)),
            CedarType::Long => Ok(syn::parse_quote!(i64)),
            CedarType::String => Ok(syn::parse_quote!(::std::string::String)),
            CedarType::Set(element_type) => {
                let sub = self.handle_sub_type(codegen, ns, in_submodule, name, element_type.as_ref())?;
                Ok(syn::parse_quote!(::std::vec::Vec<#sub>))
            }
            CedarType::Enum(variants) => Ok(self.handle_enum_type(codegen, name, variants, ns, in_submodule)),
            CedarType::Record {
                fields,
                allows_additional,
            } => self.handle_record_type(codegen, ns, name, fields, *allows_additional, in_submodule),
            CedarType::Entity {
                parents,
                shape,
                tag_type,
            } => self.handle_entity_type(codegen, ns, name, shape, tag_type.as_deref(), parents, in_submodule),
            CedarType::Reference(r) => self.handle_reference_type(codegen, r, in_submodule),
        }
    }

    /// Handles enum type generation
    fn handle_enum_type(
        &mut self,
        codegen: &Codegen,
        name: &str,
        variants: &BTreeSet<String>,
        ns: &NamespaceId,
        in_submodule: bool,
    ) -> syn::Type {
        let variants_def = variants.iter().map(|item| {
            let ident = to_upper_camel_ident(item);
            quote::quote! {
                #[serde(rename = #item)]
                #ident,
            }
        });

        let variants_match = variants.iter().map(|item| {
            let ident = to_upper_camel_ident(item);
            quote::quote! {
                Self::#ident => #item,
            }
        });

        let crate_path = &codegen.config().crate_path;

        let type_name = to_upper_camel_ident(name);
        let serde_path = format!("{}::macro_exports::serde", quote::quote!(#crate_path));
        self.items.push(syn::parse_quote! {
            #[derive(#crate_path::macro_exports::serde_derive::Serialize)]
            #[serde(crate = #serde_path)]
            pub enum #type_name {
                #(#variants_def)*
            }
        });

        let full_name = format!("{ns}::{name}").trim_start_matches("::").to_string();
        self.items.push(syn::parse_quote! {
            impl #crate_path::CedarEntity for #type_name {
                type TagType = #crate_path::NoTag;
                type Id = Self;
                type Attrs = #crate_path::NoAttributes;

                const TYPE_NAME: #crate_path::EntityTypeName = #crate_path::entity_type_name!(#full_name);

                fn entity_type_name() -> &'static #crate_path::macro_exports::cedar_policy::EntityTypeName {
                    static ENTITY_TYPE_NAME: ::std::sync::LazyLock<#crate_path::macro_exports::cedar_policy::EntityTypeName> = ::std::sync::LazyLock::new(|| {
                        std::str::FromStr::from_str(#full_name).expect("failed to parse entity type name - bug in scuffle-cedar-policy-codegen")
                    });

                    &*ENTITY_TYPE_NAME
                }
            }
        });

        self.items.push(syn::parse_quote! {
            impl #crate_path::CedarId for #type_name {
                fn into_smol_string(self) -> #crate_path::macro_exports::smol_str::SmolStr {
                    let raw = match self {
                        #(#variants_match)*
                    };

                    #crate_path::macro_exports::smol_str::SmolStr::from(raw)
                }
            }
        });

        self.items.push(syn::parse_quote! {
            impl #crate_path::CedarEnumEntity for #type_name {
                fn into_entity(self) -> #crate_path::Entity<Self>
                    where Self: Sized
                {
                    #crate_path::Entity::builder(self, #crate_path::NoAttributes).build()
                }
            }
        });

        if in_submodule {
            self.root_path
                .last()
                .map(|s| syn::parse_quote!(#s :: #type_name))
                .unwrap_or_else(|| syn::parse_quote!(#type_name))
        } else {
            syn::parse_quote!(#type_name)
        }
    }

    /// Handles record type generation
    fn handle_record_type(
        &mut self,
        codegen: &Codegen,
        ns: &NamespaceId,
        name: &str,
        fields: &BTreeMap<String, CedarTypeStructField>,
        allows_additional: bool,
        in_submodule: bool,
    ) -> CodegenResult<syn::Type> {
        if allows_additional {
            return Err(CodegenError::Unsupported("record types with additional attributes".into()));
        }

        let type_name = to_upper_camel_ident(name);
        let field_definitions = fields
            .iter()
            .map(|(field_name, field)| {
                let ident = to_snake_ident(field_name);
                let sub_type = self
                    .sub_module(field_name)
                    .handle_sub_type(codegen, ns, true, field_name, &field.ty)?;

                let mut serde_attrs = vec![quote::quote!(rename = #field_name)];
                let final_type = if field.optional {
                    serde_attrs.push(quote::quote!(skip_serializing_if = "::std::option::Option::is_none"));
                    syn::parse_quote!(::std::option::Option<#sub_type>)
                } else {
                    sub_type
                };

                Ok(quote::quote! {
                    #[serde(#(#serde_attrs),*)]
                    pub #ident: #final_type,
                })
            })
            .collect::<CodegenResult<Vec<_>>>()?;

        let crate_path = &codegen.config().crate_path;
        let serde_path = format!("{}::macro_exports::serde", quote::quote!(#crate_path));

        self.items.push(syn::parse_quote! {
            #[derive(#crate_path::macro_exports::serde_derive::Serialize)]
            #[serde(crate = #serde_path)]
            pub struct #type_name {
                #(#field_definitions)*
            }
        });

        Ok(if in_submodule {
            self.root_path
                .last()
                .map(|s| syn::parse_quote!(#s :: #type_name))
                .unwrap_or_else(|| syn::parse_quote!(#type_name))
        } else {
            syn::parse_quote!(#type_name)
        })
    }

    /// Handles entity type generation
    #[allow(clippy::too_many_arguments)]
    fn handle_entity_type(
        &mut self,
        codegen: &Codegen,
        ns: &NamespaceId,
        name: &str,
        shape: &CedarType,
        tag_type: Option<&CedarType>,
        parents: &[CedarRef],
        in_submodule: bool,
    ) -> CodegenResult<syn::Type> {
        let path = self.handle_sub_type(codegen, ns, false, name, shape)?;
        let crate_path = &codegen.config().crate_path;

        let tag_type = tag_type
            .as_ref()
            .map(|tag_type| {
                self.sub_module(name)
                    .handle_sub_type(codegen, ns, true, "EntityTag", tag_type)
            })
            .unwrap_or_else(|| Ok(syn::parse_quote!(#crate_path::NoTag)))?;

        let full_name = format!("{ns}::{name}").trim_start_matches("::").to_string();

        self.items.push(syn::parse_quote! {
            impl #crate_path::CedarEntity for #path {
                type TagType = #tag_type;
                type Id = ::std::string::String;
                type Attrs = Self;

                const TYPE_NAME: #crate_path::EntityTypeName = #crate_path::entity_type_name!(#full_name);

                fn entity_type_name() -> &'static #crate_path::macro_exports::cedar_policy::EntityTypeName {
                    static ENTITY_TYPE_NAME: ::std::sync::LazyLock<#crate_path::macro_exports::cedar_policy::EntityTypeName> = ::std::sync::LazyLock::new(|| {
                        std::str::FromStr::from_str(#full_name).expect("failed to parse entity type name - bug in scuffle-cedar-policy-codegen")
                    });

                    &*ENTITY_TYPE_NAME
                }
            }
        });

        for parent in parents {
            match codegen.resolve_ref(parent) {
                None => return Err(CodegenError::UnresolvedReference(parent.to_string())),
                Some(p) if !p.is_entity() => {
                    return Err(CodegenError::ExpectedEntity {
                        common_type: parent.to_string(),
                        ty: format!("entity {name}"),
                    });
                }
                Some(_) => {}
            }

            let parent_ty = find_relative_path(&self.root_path, &parent.ident_path());
            self.items.push(syn::parse_quote! {
                impl #crate_path::CedarChild<#parent_ty> for #path {}
            });
        }

        Ok(if in_submodule { syn::parse_quote!(super::#path) } else { path })
    }

    /// Handles reference type generation
    pub(crate) fn handle_reference_type(
        &self,
        codegen: &Codegen,
        r: &CedarRef,
        in_submodule: bool,
    ) -> CodegenResult<syn::Type> {
        let relative = if in_submodule {
            &self.root_path[..self.root_path.len() - 1]
        } else {
            &self.root_path
        };

        let path = find_relative_path(relative, &r.ident_path());
        let Some(reference) = codegen.resolve_ref(r) else {
            return Err(CodegenError::UnresolvedReference(r.to_string()));
        };

        let crate_path = &codegen.config().crate_path;

        if reference.is_entity() {
            Ok(syn::parse_quote!(#crate_path::EntityUid<#path>))
        } else {
            Ok(syn::parse_quote!(#path))
        }
    }

    /// Handles top-level type generation
    pub(crate) fn handle_type(
        &mut self,
        codegen: &Codegen,
        ns: &NamespaceId,
        name: impl AsRef<str>,
        ty: &CedarType,
    ) -> CodegenResult<()> {
        match ty {
            CedarType::Bool | CedarType::Long | CedarType::String => {
                let type_name = to_upper_camel_ident(name.as_ref());
                let sub_type = self.handle_sub_type(codegen, ns, false, name, ty)?;
                self.items.push(syn::parse_quote! {
                    pub type #type_name = #sub_type;
                });
            }
            CedarType::Set(_) => {
                let type_name = to_upper_camel_ident(name.as_ref());
                let sub_type = self.sub_module(name).handle_sub_type(codegen, ns, true, "SetInner", ty)?;
                self.items.push(syn::parse_quote! {
                    type #type_name = #sub_type;
                });
            }
            CedarType::Reference(_) => {
                let type_name = to_upper_camel_ident(name.as_ref());
                let sub_type = self.handle_sub_type(codegen, ns, false, name, ty)?;
                self.items.push(syn::parse_quote! {
                    type #type_name = #sub_type;
                });
            }
            ty => {
                self.handle_sub_type(codegen, ns, false, name, ty)?;
            }
        }

        Ok(())
    }

    pub(crate) fn handle_action(
        &mut self,
        codegen: &Codegen,
        ns_id: &NamespaceId,
        action: &str,
        ty: &CedarAction,
    ) -> CodegenResult<(), CodegenError> {
        let ident = to_upper_camel_ident(action);

        // Generate action struct
        self.items.push(syn::parse_quote! {
            pub struct #ident;
        });

        let crate_path = &codegen.config().crate_path;

        let ty_name = CedarRef {
            id: Id::from_str("Action").unwrap(),
            namespace: ns_id.clone(),
        }
        .to_string();

        // Generate Serialize implementation
        self.items.push(syn::parse_quote! {
            impl #crate_path::CedarActionEntity for #ident {
                fn action_entity_uid() -> &'static #crate_path::macro_exports::cedar_policy::EntityUid {
                    static ENTITY_UID: ::std::sync::LazyLock<#crate_path::macro_exports::cedar_policy::EntityUid> = ::std::sync::LazyLock::new(|| {
                        #crate_path::macro_exports::cedar_policy::EntityUid::from_type_name_and_id(
                            std::str::FromStr::from_str(#ty_name).expect("failed to parse euid - bug in scuffle-cedar-policy-codegen"),
                            std::str::FromStr::from_str(#action).expect("failed to parse euid - bug in scuffle-cedar-policy-codegen"),
                        )
                    });

                    &*ENTITY_UID
                }
            }
        });

        for parent in &ty.parents {
            if !codegen.contains_action(ns_id, parent) {
                return Err(CodegenError::UnresolvedReference(parent.to_string()));
            }

            let parent_ident = to_upper_camel_ident(&parent.name);
            let parent_path = if let Some(pid) = &parent.id {
                let idents = pid
                    .namespace
                    .items
                    .iter()
                    .chain(std::iter::once(&pid.id))
                    .map(to_snake_ident)
                    .chain(std::iter::once(parent_ident))
                    .collect::<Vec<_>>();

                find_relative_path(&self.root_path, &idents)
            } else {
                syn::parse_quote!(#parent_ident)
            };

            self.items.push(syn::parse_quote! {
                impl #crate_path::CedarChild<#parent_path> for #ident {}
            });
        }

        // Generate context type
        let ctx = ty
            .context
            .as_ref()
            .map(|ctx| self.sub_module(action).handle_sub_type(codegen, ns_id, true, "Context", ctx))
            .unwrap_or_else(|| Ok(syn::parse_quote!(#crate_path::EmptyContext)))?;

        // Generate action implementations
        let resolve_types = |reference| match codegen.resolve_ref(reference) {
            None => Err(CodegenError::UnresolvedReference(reference.to_string())),
            Some(r) if !r.is_entity() => Err(CodegenError::ExpectedEntity {
                common_type: reference.to_string(),
                ty: format!("action {action}"),
            }),
            Some(_) => Ok(find_relative_path(&self.root_path, &reference.ident_path())),
        };

        let principals = ty.principals.iter().map(resolve_types).collect::<CodegenResult<Vec<_>>>()?;
        let resources = ty.resources.iter().map(resolve_types).collect::<CodegenResult<Vec<_>>>()?;

        for principal in principals {
            for resource in &resources {
                self.items.push(syn::parse_quote! {
                    impl #crate_path::CedarAction<#principal, #resource> for #ident {
                        type Context = #ctx;
                    }
                });
            }
        }

        Ok(())
    }
}
