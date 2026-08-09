#![allow(missing_docs)]

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("{}", args.path.display())
}
