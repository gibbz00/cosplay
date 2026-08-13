use async_wayland_xml::{Protocol, utils::EnumReprMapBuildError};

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
    /// Convert a wayland protocol into Rust source code implementing `async-wayland-codec`
    /// traits.
    pub fn run(protocol: Protocol, config: GeneratorConfig) -> Result<String, GeneratorError> {
        let src = ProtocolItem::quote(protocol, config)?;

        let string = Formatter::format(src).expect("Failed to parse src tokens.");

        Ok(string)
    }
}
