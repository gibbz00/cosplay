//! Items for modifying generator output.

mod core;
pub use core::GeneratorConfig;

mod external_interfaces;
pub use external_interfaces::ExternalInterfaces;

mod name_mappings;
pub use name_mappings::{ItemType, NameMappings};
