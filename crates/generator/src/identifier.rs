use async_wayland_xml::EnumPath;
use heck::{ToSnakeCase, ToUpperCamelCase};
use quote::quote;

pub struct IdentifierItem;

impl IdentifierItem {
    pub fn type_name(name: impl AsRef<str>) -> proc_macro2::Ident {
        quote::format_ident!("{}", name.as_ref().to_upper_camel_case())
    }

    pub fn field_name(name: impl AsRef<str>) -> proc_macro2::Ident {
        quote::format_ident!("{}", name.as_ref().to_snake_case())
    }

    pub fn module_name(name: impl AsRef<str>) -> proc_macro2::Ident {
        quote::format_ident!("{}", name.as_ref().to_snake_case())
    }

    pub fn qualified_interface(name: impl AsRef<str>) -> proc_macro2::TokenStream {
        let str = name.as_ref();

        let module_name = Self::module_name(str);

        let type_name = Self::type_name(str);

        quote! { crate::#module_name::#type_name }
    }

    pub fn enum_path(path: EnumPath) -> proc_macro2::TokenStream {
        let EnumPath { interface, enumeration } = path;

        let name = Self::type_name(enumeration);

        match interface {
            Some(interface_name) => {
                let module = Self::module_name(interface_name);
                quote! { crate::#module::#name }
            }
            None => quote! { #name },
        }
    }
}
