use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use structopt::StructOpt;

mod schema;

#[derive(StructOpt)]
#[structopt(name = env!("CARGO_PKG_NAME"))]
#[structopt(version = env!("CARGO_PKG_VERSION"))]
#[structopt(author = env!("CARGO_PKG_AUTHORS"))]
#[structopt(about = env!("CARGO_PKG_DESCRIPTION"))]
struct Args {
    /// The input file
    input:  PathBuf,
    /// The output file
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::from_args();

    Ok(())
}
