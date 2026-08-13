//! Small utility crate for producing `cosplay-codec` implementations from Wayland XML protocol
//! specifications.
//!
//! Each build script would normally need to manually pull in and hook together a config
//! deserializer (i.e. `toml`), `cosplay-xml`, and finally `cosplay-generator`. `cosplay-scanner`
//! wraps all of this in [`run()`].

use std::path::Path;

use anyhow::Context;

/// Returns the corresponding `cosplay-codec` integrated Rust source from the provided XML path.
///
/// Uses:
///
/// * `cosplay_xml` for deserializing the XML string contents at `xml_path`.
/// * `toml` for deserializing the TOML-formatted [GeneratorConfig] at `config_path`.
/// * `cosplay_generator` for generating the `cosplay-codec` implementations.
///
/// [GeneratorConfig]: [cosplay_generator::config::GeneratorConfig]
pub fn run(xml_path: &Path, config_path: Option<&Path>) -> anyhow::Result<String> {
    let protocol = {
        let xml = std::fs::read_to_string(xml_path).context(format!("Unable to read XML at {}.", xml_path.display()))?;
        cosplay_xml::Protocol::from_xml(&xml).context("Failed to deserialize XML.")?
    };

    let config = match config_path {
        Some(path) => {
            let config_str = std::fs::read_to_string(path).context(format!("Unable to read config at {}.", path.display()))?;
            toml::from_str(&config_str).context("Failed to deserialize config.")?
        }
        None => Default::default(),
    };

    cosplay_generator::Generator::run(protocol, config).context("Failed to generate source code.")
}
