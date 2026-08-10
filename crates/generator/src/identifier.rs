use async_wayland_xml::{Cname, CnameSuffix, EnumPath};
use heck::{ToSnakeCase, ToUpperCamelCase};
use quote::quote;

pub struct IdentifierItem;

impl IdentifierItem {
    pub fn type_name(name: &Cname) -> proc_macro2::Ident {
        quote::format_ident!("{}", name.as_ref().to_upper_camel_case())
    }

    /// Workaround for how wayland made the decision to allow enum names and
    /// variants to begin with digits :/
    pub fn sanitized_type_name(name: &CnameSuffix) -> proc_macro2::Ident {
        let ident = name.as_ref().to_upper_camel_case();

        match name.as_ref().chars().next().is_some_and(|char| char.is_ascii_digit()) {
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

    pub fn enum_path(path: EnumPath) -> proc_macro2::TokenStream {
        let EnumPath { interface, enumeration } = path;

        let name = Self::sanitized_type_name(&enumeration);

        match interface {
            Some(interface_name) => {
                let module = Self::module_name(&interface_name);
                quote! { crate::#module::#name }
            }
            None => quote! { #name },
        }
    }
}
