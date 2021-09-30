use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use structopt::StructOpt;
use traffloat::save;

#[derive(StructOpt)]
#[structopt(name = env!("CARGO_PKG_NAME"))]
#[structopt(version = env!("CARGO_PKG_VERSION"))]
#[structopt(author = env!("CARGO_PKG_AUTHORS"))]
#[structopt(about = env!("CARGO_PKG_DESCRIPTION"))]
enum Args {
    /// Converts a save file to binary format.
    ToBinary {
        /// The input file
        input:  PathBuf,
        /// The output file
        output: Option<PathBuf>,
    },
    /// Converts a save file to text format.
    ToText {
        /// The input file
        input:  PathBuf,
        /// The output file
        output: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::from_args();

    match &args {
        Args::ToBinary { input, output } => {
            let output = match output {
                Some(file) => file.clone(),
                None => input.with_extension("tsv"),
            };
            convert(input, &output, save::Format::Binary)?;
        }
        Args::ToText { input, output } => {
            let output = match output {
                Some(file) => file.clone(),
                None => input.with_extension("tsvt"),
            };
            convert(input, &output, save::Format::Text)?;
        }
    }

    Ok(())
}

fn convert(input: &Path, output: &Path, format: save::Format) -> anyhow::Result<()> {
    let read = fs::read(&input).context("Error reading input file")?;
    let object = save::parse(&read).context("Error parsing input file")?;
    let write = save::emit(&object, &save::Request::builder().format(format).build())
        .context("Error encoding output file")?;
    fs::write(output, write).context("Error writing output file")?;

    Ok(())
}
