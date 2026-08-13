use cosplay_xml::{Protocol, utils::EnumReprMap};
use quote::quote;

use crate::*;

pub struct ProtocolItem;

impl ProtocolItem {
    pub fn quote(protocol: Protocol, config: GeneratorConfig) -> Result<proc_macro2::TokenStream, GeneratorError> {
        let repr_map = EnumReprMap::new(&protocol)?;

        let Protocol { description, interfaces, .. } = protocol;

        let description_comment = DocumentationItem::quote_inner(description.as_ref());

        let interface_modules = interfaces.into_iter().map(|interface| {
            InterfaceItem::quote(
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

#[cfg(test)]
mod tests {
    use cosplay_xml::{Cname, Interface};

    use super::*;

    #[test]
    fn module_per_interface() {
        let mut protocol = Protocol::new(Cname::parse("protocol".to_string()).unwrap());
        protocol.interfaces = vec![
            Interface::new(Cname::parse("wl_a".to_string()).unwrap(), Default::default()),
            Interface::new(Cname::parse("wl_b".to_string()).unwrap(), Default::default()),
        ];

        let quote = ProtocolItem::quote(protocol, Default::default()).unwrap();

        let actual = Generator::format(quote);

        let expected = indoc::indoc! {"
            pub mod wl_a {
                pub struct WlA;
            }
            pub mod wl_b {
                pub struct WlB;
            }
        "};

        assert_eq!(expected, actual);
    }
}
