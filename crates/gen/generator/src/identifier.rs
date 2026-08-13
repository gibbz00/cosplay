use cosplay_xml::{Cname, CnameSuffix, EnumPath};
use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use quote::quote;

use crate::*;

pub struct IdentifierItem;

impl IdentifierItem {
    pub fn type_name(name: &Cname) -> proc_macro2::Ident {
        quote::format_ident!("{}", name.as_ref().to_upper_camel_case())
    }

    pub fn sanitized_type_name(name: &CnameSuffix) -> proc_macro2::Ident {
        let ident = name.as_ref().to_upper_camel_case();
        Self::sanitized_ident(&ident)
    }

    pub fn sanitized_const_name(name: &CnameSuffix) -> proc_macro2::Ident {
        let ident = name.as_ref().to_shouty_snake_case();
        Self::sanitized_ident(&ident)
    }

    /// Workaround for how wayland made the decision to allow
    /// enum names and variants to begin with digits :/
    fn sanitized_ident(ident: &str) -> proc_macro2::Ident {
        match ident.chars().next().is_some_and(|char| char.is_ascii_digit()) {
            true => quote::format_ident!("_{ident}"),
            false => quote::format_ident!("{ident}"),
        }
    }

    pub fn field_name(name: &Cname) -> proc_macro2::Ident {
        quote::format_ident!("{}", name.as_ref().to_snake_case())
    }

    pub fn module_name(name: &Cname) -> proc_macro2::Ident {
        quote::format_ident!("{}", name.as_ref().to_snake_case())
    }

    pub fn qualified_interface(name: &Cname) -> proc_macro2::TokenStream {
        let module_name = Self::module_name(name);

        let type_name = Self::type_name(name);

        quote! { crate::#module_name::#type_name }
    }

    pub fn enum_path(parent_interface: &Cname, path: EnumPath, name_mappings: &NameMappings) -> proc_macro2::TokenStream {
        let EnumPath { interface, enumeration } = path;

        let interface_name = interface.as_ref().unwrap_or(parent_interface);

        let mapped_name = name_mappings
            .get(interface_name, &enumeration, ItemType::Enum)
            .unwrap_or(&enumeration);

        let name = Self::sanitized_type_name(mapped_name);

        match interface {
            Some(interface_name) => {
                let module = Self::module_name(&interface_name);
                quote! { crate::#module::#name }
            }
            None => quote! { #name },
        }
    }
}
