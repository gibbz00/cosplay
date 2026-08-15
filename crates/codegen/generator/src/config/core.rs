use crate::*;

/// Configuration for modifying generator output.
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct GeneratorConfig {
    /// Register crate paths for external interfaces.
    #[serde(default)]
    pub external_interfaces: ExternalInterfaces,

    /// Modify generated item names.
    #[serde(default)]
    pub name_mappings: NameMappings,
}
