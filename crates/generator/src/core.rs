use anyhow::Context;
use async_wayland_xml::{Protocol, utils::EnumReprMap};
use quote::quote;

use crate::*;

/// Use with [`Generator::run`].
pub struct Generator;

impl Generator {
    /// Convert a wayland protocol into Rust source code implementing `async-wayland-codec`
    /// traits.
    pub fn run(protocol: Protocol, config: GeneratorConfig) -> anyhow::Result<String> {
        let repr_map = EnumReprMap::new(&protocol).context("Failed to build enum representation map.")?;

        let Protocol { description, interfaces, .. } = protocol;

        let description_comment = Documentation::quote_inner(description.as_ref());

        let interface_modules = interfaces.into_iter().map(|interface| {
            InterfaceModule::quote(
                interface,
                InterfaceContext { repr_map: &repr_map, name_mappings: &config.name_mappings },
            )
        });

        let src = quote! {
            #description_comment

            #(#interface_modules)*
        };

        Formatter::format(src).context("Failed to parse src tokens.")
    }
}
