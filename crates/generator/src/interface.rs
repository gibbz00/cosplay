use async_wayland_xml::{Interface, utils::EnumReprMap};
use quote::quote;

use crate::*;

pub struct InterfaceModule;

impl InterfaceModule {
    pub fn quote(interface: Interface, repr_map: &EnumReprMap) -> proc_macro2::TokenStream {
        let Interface { name, version, frozen, description, requests, events, enums } = interface;

        let module_name = IdentifierItem::module_name(&name);

        let item_name = IdentifierItem::type_name(&name);

        let doc = Documentation::quote_outer(description.as_ref());

        let requests = MessageItem::quote_list(requests).collect::<Vec<_>>();

        let events = MessageItem::quote_list(events);

        let enums = EnumItem::quote_list(&name, enums, repr_map);

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
