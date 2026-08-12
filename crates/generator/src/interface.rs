use async_wayland_xml::{Interface, utils::EnumReprMap};
use quote::quote;

use crate::*;

pub struct InterfaceModule;

#[derive(Clone, Copy)]
pub struct InterfaceContext<'a> {
    pub repr_map: &'a EnumReprMap,
    pub name_mappings: &'a NameMappings,
}

impl InterfaceModule {
    pub fn quote(interface: Interface, ctx: InterfaceContext) -> proc_macro2::TokenStream {
        let Interface { name, version, frozen, description, requests, events, enums } = interface;

        let module_name = IdentifierItem::module_name(&name);

        let item_name = IdentifierItem::type_name(&name);

        let doc = Documentation::quote_outer(description.as_ref());

        let message_ctx = MessageContext { name_mappings: ctx.name_mappings };

        let requests = MessageItem::quote_list(requests, message_ctx);

        let events = MessageItem::quote_list(events, message_ctx);

        let enums = EnumItem::quote_list(
            enums,
            EnumContext {
                interface_name: &name,
                repr_map: ctx.repr_map,
                name_mappings: ctx.name_mappings,
            },
        );

        quote! {
            pub mod #module_name {
                #doc
                pub struct #item_name;

                #(#requests)*

                #(#events)*

                #(#enums)*
            }
        }
    }
}
