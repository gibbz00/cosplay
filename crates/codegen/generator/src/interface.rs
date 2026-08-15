use cosplay_xml::{Cname, Interface};
use quote::quote;

use crate::*;

pub struct InterfaceItem;

#[derive(Clone, Copy)]
pub struct InterfaceContext<'a> {
    pub external_interfaces: &'a ExternalInterfaces,
    pub name_mappings: &'a NameMappings,
}

impl InterfaceItem {
    pub fn quote(interface: Interface, ctx: InterfaceContext) -> proc_macro2::TokenStream {
        let Interface { name, description, requests, events, enums, .. } = interface;

        let module_name = IdentifierItem::module_name(&name);

        let doc = DocumentationItem::quote_outer(description.as_ref());

        let ident = IdentifierItem::type_name(&name);

        let trait_impl = Self::trait_impl(&name, &ident);

        let message_ctx = MessageContext {
            interface_name: &name,
            external_interfaces: ctx.external_interfaces,
            name_mappings: ctx.name_mappings,
        };

        let requests = MessageItem::quote_list(requests, message_ctx);

        let events = MessageItem::quote_list(events, message_ctx);

        let enums = EnumItem::quote_list(enums, EnumContext { interface_name: &name, name_mappings: ctx.name_mappings });

        quote! {
            pub mod #module_name {
                #doc
                pub struct #ident;

                #trait_impl

                #(#requests)*

                #(#events)*

                #(#enums)*
            }
        }
    }

    fn trait_impl(name: &Cname, ident: &proc_macro2::Ident) -> proc_macro2::TokenStream {
        let name = name.as_ref();
        quote! {
            impl ::cosplay_codec::Interface for #ident {
                const NAME: &str = #name;
            }
        }
    }
}
