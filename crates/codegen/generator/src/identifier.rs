use cosplay_xml::{Cname, CnameSuffix};
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

    pub fn qualified_interface(external_interfaces: &ExternalInterfaces, interface_name: &Cname) -> proc_macro2::TokenStream {
        let interface_ident = Self::type_name(interface_name);
        Self::qualified_interface_item(external_interfaces, interface_name, &interface_ident)
    }

    pub fn qualified_interface_item(
        external_interfaces: &ExternalInterfaces,
        interface_name: &Cname,
        item_name: &proc_macro2::Ident,
    ) -> proc_macro2::TokenStream {
        let qualifier = match external_interfaces.get(interface_name) {
            Some(path) => {
                quote! { #path }
            }
            None => quote! { crate },
        };

        let module_name = Self::module_name(interface_name);

        quote! { #qualifier::#module_name::#item_name }
    }
}
