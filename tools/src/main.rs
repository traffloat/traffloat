use std::fs;
use std::io::{self, BufReader, BufWriter};
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use rootcause::prelude::ResultExt;

#[derive(clap::Parser)]
struct Command {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    #[clap(alias = "t2j")]
    TfsaveToJson {
        #[clap(default_value = "-")]
        input:  PathBuf,
        #[clap(default_value = "-")]
        output: PathBuf,
    },
}

fn main() -> rootcause::Result<()> {
    let command = <Command as clap::Parser>::parse();

    match command.subcommand {
        Subcommand::TfsaveToJson { input, output } => {
            let read = if input == Path::new("-") {
                Box::new(BufReader::new(io::stdin())) as Box<dyn io::Read>
            } else {
                Box::new(BufReader::new(
                    fs::File::open(input).context("Failed to open input file")?,
                ))
            };
            let read = BufReader::new(read);
            let read = zstd::Decoder::new(read).context("File is not zstd compressed")?;
            let entries: Vec<(String, Vec<u8>)> =
                ciborium::from_reader(read).context("CBOR format error")?;

            let json_entries: IndexMap<String, ciborium::Value> = entries
                .into_iter()
                .map(|(key, value)| -> rootcause::Result<_> {
                    {
                        let value: ciborium::Value =
                            ciborium::from_reader(value.as_slice()).context("CBOR format error")?;
                        rootcause::Result::Ok((key.clone(), value))
                    }
                    .attach(key)
                })
                .collect::<rootcause::Result<_, _>>()?;

            let write = if output == Path::new("-") {
                Box::new(BufWriter::new(io::stdout())) as Box<dyn io::Write>
            } else {
                Box::new(BufWriter::new(
                    fs::File::create(output).context("Failed to create output file")?,
                ))
            };
            let write = BufWriter::new(write);
            serde_json::to_writer(write, &json_entries).context("Failed to write JSON output")?;
        }
    }

    Ok(())
}
