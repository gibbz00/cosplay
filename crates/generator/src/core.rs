use anyhow::Context;
use async_wayland_xml::Protocol;
use quote::quote;

use crate::*;

/// Use with [`Generator::run`].
pub struct Generator;

impl Generator {
    /// Convert a wayland protocol into Rust source code implementing `async-wayland-codec`
    /// traits.
    pub fn run(protocol: Protocol) -> anyhow::Result<String> {
        let Protocol { description, interfaces, .. } = protocol;

        let description_comment = Documentation::quote_inner(description);

        let interface_modules = interfaces.into_iter().map(InterfaceModule::quote);

        let src = quote! {
            #description_comment

            #(#interface_modules)*
        };

        Formatter::format(src).context("Failed to parse src tokens.")
    }
}
