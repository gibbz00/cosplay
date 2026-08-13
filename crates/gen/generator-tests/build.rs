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

        let config_path = {
            let name = path
                .file_stem()
                .expect("Path has no file name.")
                .to_str()
                .expect("File name is valid unicode.");

            let toml_path = Path::new(TOML_DIR).join(format!("{name}.toml"));

            toml_path.exists().then_some(toml_path)
        };

        let rust = cosplay_scanner::run(&path, config_path.as_deref()).expect("Failed to run scanner.");

        output_file.write_all(rust.as_bytes())?;
    }

    Ok(())
}
