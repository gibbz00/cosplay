use cosplay_xml::{Protocol, utils::EnumReprMapBuildError};

use crate::*;

/// Use with [`Generator::run`].
pub struct Generator;

/// Returned from [`Generator::run`]
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum GeneratorError {
    #[error("Failed to build enum representation map: {0}")]
    EnumReprMap(#[from] EnumReprMapBuildError),
}

impl Generator {
    /// Convert a wayland protocol into Rust source code implementing `cosplay-codec`
    /// traits.
    pub fn run(protocol: Protocol, config: GeneratorConfig) -> Result<String, GeneratorError> {
        ProtocolItem::quote(protocol, config).map(Self::format)
    }

    pub(crate) fn format(src: proc_macro2::TokenStream) -> String {
        syn::parse2::<syn::File>(src)
            .map(|file| prettyplease::unparse(&file))
            .expect("Failed to parse src tokens.")
    }
}
