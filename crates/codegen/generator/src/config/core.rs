use crate::*;

/// Configuration for modifying generator output.
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct GeneratorConfig {
    /// Modify generated item names.
    pub name_mappings: NameMappings,
}
