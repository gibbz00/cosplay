use async_wayland_xml::Interface;
use quote::quote;

use crate::*;

pub struct InterfaceModule;

impl InterfaceModule {
    pub fn quote(interface: Interface) -> proc_macro2::TokenStream {
        let Interface { name, version, frozen, description, requests, events, enums } = interface;

        let module_name = IdentifierItem::module_name(&name);

        let item_name = IdentifierItem::type_name(name);

        let doc = Documentation::quote_outer(description);

        let requests = MessageItem::quote_list(requests);

        let events = MessageItem::quote_list(events);

        quote! {
            pub mod #module_name {
                #doc
                pub struct #item_name;

                #(#requests)*

                #(#events)*
            }
        }
    }
}
