use async_wayland_xml::Interface;
use quote::quote;

use crate::*;

pub struct InterfaceModule;

impl InterfaceModule {
    pub fn quote(interface: Interface) -> proc_macro2::TokenStream {
        let Interface { name, version, frozen, description, requests, events, enums } = interface;

        let name = quote::format_ident!("{}", name.as_ref());
        let description_comment = Documentation::quote_outer(description);

        quote! {
            pub mod #name {
                #description_comment
            }
        }
    }
}
