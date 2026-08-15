use cosplay_xml::Protocol;
use quote::quote;

use crate::*;

pub struct ProtocolItem;

impl ProtocolItem {
    pub fn quote(protocol: Protocol, config: GeneratorConfig) -> proc_macro2::TokenStream {
        let Protocol { description, interfaces, .. } = protocol;

        let GeneratorConfig { external_interfaces, name_mappings } = &config;

        let description_comment = DocumentationItem::quote_inner(description.as_ref());

        let interface_modules = interfaces
            .into_iter()
            .map(|interface| InterfaceItem::quote(interface, InterfaceContext { external_interfaces, name_mappings }));

        quote! {
            #description_comment

            #(#interface_modules)*
        }
    }
}
