use async_wayland_xml::Interface;
use quote::quote;

use crate::*;

pub struct InterfaceModule;

impl InterfaceModule {
    pub fn quote(interface: Interface) -> proc_macro2::TokenStream {
        let Interface { name, version, frozen, description, requests, events, enums } = interface;

        let name = IdentifierItem::module_name(name);

        let description_comment = Documentation::quote_inner(description);

        let requests = MessageItem::quote_list(requests);

        let events = MessageItem::quote_list(events);

        quote! {
            pub mod #name {
                #description_comment

                #(#requests)*

                #(#events)*
            }
        }
    }
}
