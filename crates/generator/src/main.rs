#![allow(missing_docs)]

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let xml = std::fs::read_to_string(&args.path)?;

    let protocol = async_wayland_xml::Protocol::from_xml(&xml)?;

    let src = async_wayland_generator::Generator::run(protocol)?;

    println!("{src}");

    Ok(())
}
