use async_wayland_xml::{Protocol, utils::EnumReprMap};
use quote::quote;

use crate::*;

pub struct ProtocolItem;

impl ProtocolItem {
    pub fn quote(protocol: Protocol, config: GeneratorConfig) -> Result<proc_macro2::TokenStream, GeneratorError> {
        let repr_map = EnumReprMap::new(&protocol)?;

        let Protocol { description, interfaces, .. } = protocol;

        let description_comment = Documentation::quote_inner(description.as_ref());

        let interface_modules = interfaces.into_iter().map(|interface| {
            InterfaceModule::quote(
                interface,
                InterfaceContext { repr_map: &repr_map, name_mappings: &config.name_mappings },
            )
        });

        Ok(quote! {
            #description_comment

            #(#interface_modules)*
        })
    }
}
