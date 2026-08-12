#![allow(missing_docs)]

use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    path: PathBuf,

    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let protocol = {
        let xml = std::fs::read_to_string(&args.path).context(format!("Unable to read XML at {}.", args.path.display()))?;
        async_wayland_xml::Protocol::from_xml(&xml).context("Failed to deserialize XML.")?
    };

    let config = match args.config {
        None => Default::default(),
        Some(path) => {
            let config_str = std::fs::read_to_string(&path).context(format!("Unable to read config at {}.", path.display()))?;
            toml::from_str(&config_str).context("Failed to deserialize config.")?
        }
    };

    let src = async_wayland_generator::Generator::run(protocol, config)?;

    println!("{src}");

    Ok(())
}
