#![allow(missing_docs)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=./input");

    let src = cosplay_scanner::run("./input/protocol.xml", Some("./input/config.toml")).unwrap();

    let out_dir = std::env::var_os("OUT_DIR").unwrap();

    let dest_path = std::path::Path::new(&out_dir).join("generated.rs");

    std::fs::write(dest_path, src).unwrap();

    Ok(())
}
