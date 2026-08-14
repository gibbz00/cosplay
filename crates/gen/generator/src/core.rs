use cosplay_xml::Protocol;

use crate::*;

/// Use with [`Generator::run`].
pub struct Generator;

impl Generator {
    /// Convert a wayland protocol into Rust source code implementing `cosplay-codec`
    /// traits.
    pub fn run(protocol: Protocol, config: GeneratorConfig) -> String {
        let tokens = ProtocolItem::quote(protocol, config);
        Self::format(tokens)
    }

    pub(crate) fn format(src: proc_macro2::TokenStream) -> String {
        syn::parse2::<syn::File>(src)
            .map(|file| prettyplease::unparse(&file))
            .expect("Failed to parse src tokens.")
    }
}
