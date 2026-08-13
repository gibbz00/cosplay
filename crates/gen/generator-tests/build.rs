#![allow(missing_docs)]

use std::{io::Write, path::Path};

const XML_DIR: &str = "xml";
const TOML_DIR: &str = "config";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed={XML_DIR}");
    println!("cargo::rerun-if-changed={TOML_DIR}");

    let mut output_file = {
        let out_dir = std::env::var_os("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("combined.rs");
        std::fs::File::create(dest_path)?
    };

    for entry in std::fs::read_dir(XML_DIR)? {
        let path = entry?.path();

        let xml = std::fs::read_to_string(&path)?;

        let protocol = cosplay_xml::Protocol::from_xml(&xml)?;

        let config = {
            let name = path.file_stem().expect("Path has no file name.");
            read_toml_config(name)?.unwrap_or_default()
        };

        let rust = cosplay_generator::Generator::run(protocol, config)?;

        output_file.write_all(rust.as_bytes())?;
    }

    Ok(())
}

fn read_toml_config(
    name: &std::ffi::OsStr,
) -> Result<Option<cosplay_generator::config::GeneratorConfig>, Box<dyn std::error::Error>> {
    let name = name.to_str().expect("File name is valid unicode.");

    let toml_path = Path::new(TOML_DIR).join(format!("{name}.toml"));

    if !toml_path.exists() {
        return Ok(None);
    }

    let toml_str = std::fs::read_to_string(&toml_path)?;

    let config = toml::from_str(&toml_str)?;

    Ok(Some(config))
}
