use std::path::{Path, PathBuf};
use std::{fs, io};

use anyhow::{Context as _, Result};
use codegen::{Definition, ResolveContext};
use def::Schema;
use structopt::StructOpt;
use traffloat_def::{self as def, Def};

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

fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = Args::from_args();

    let mut defs = Vec::new();
    let mut context = ResolveContext::default();
    {
        context.start_tracking::<def::lang::Def>();
        context.start_tracking::<def::atlas::Def>();
        context.start_tracking::<def::liquid::Def>();
        context.start_tracking::<def::gas::Def>();
        context.start_tracking::<def::cargo::category::Def>();
        context.start_tracking::<def::cargo::Def>();
        context.start_tracking::<def::skill::Def>();
        context.start_tracking::<def::vehicle::Def>();
        context.start_tracking::<def::building::category::Def>();
        context.start_tracking::<def::building::Def>();
        context.start_tracking::<def::crime::Def>();
    }

    let schema::Main { scenario, config } =
        read_main_defs(&mut defs, &mut context, &args.input).context("Parsing input files")?;

    let schema = Schema::builder().scenario(scenario).config(config).def(defs).build();

    write(&args.output, schema).context("Saving output")?;

    Ok(())
}

fn read_main_defs(
    defs: &mut Vec<Def>,
    context: &mut ResolveContext,
    path: &Path,
) -> Result<schema::Main> {
    let string = fs::read_to_string(path).with_context(|| format!("Reading {}", path.display()))?;
    let mut de = toml::Deserializer::new(&string);
    let toml: schema::MainFile = serde_path_to_error::deserialize(&mut de)
        .with_context(|| format!("Parsing {}", path.display()))?;

    read_defs(defs, context, toml.file, path)?;

    Ok(toml.main)
}

fn read_included_defs(
    defs: &mut Vec<Def>,
    context: &mut ResolveContext,
    path: &Path,
) -> Result<()> {
    let string = fs::read_to_string(path).with_context(|| format!("Reading {}", path.display()))?;
    let mut de = toml::Deserializer::new(&string);
    let toml: schema::File = serde_path_to_error::deserialize(&mut de)
        .with_context(|| format!("Parsing {}", path.display()))?;

    read_defs(defs, context, toml, path)
}

fn read_defs(
    defs: &mut Vec<Def>,
    context: &mut ResolveContext,
    file: schema::File,
    path: &Path,
) -> Result<()> {
    for include in file.include {
        let path = path
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize {}", path.display()))?;
        let dir = path.parent().context("Regular file has no parent")?;

        let mut included = dir.join(&include.file);
        included = included.canonicalize().with_context(|| {
            format!("Failed to canonicalize included file {}", included.display())
        })?;
        read_included_defs(defs, context, &included)
            .with_context(|| format!("Included in {}", path.display()))?;
    }

    log::debug!("Loading {}", path.display());

    for def in file.def {
        let def = Def::convert(def, context)
            .with_context(|| format!("Resolving references in {}", path.display()))?;
        defs.push(def);
    }

    Ok(())
}

fn write(dir: &Path, schema: Schema) -> Result<()> {
    fs::create_dir(dir).context("Creating output directory")?;

    let scenario = dir.join("scenario.tfsave");

    let file = fs::File::create(&scenario).context("Writing scenario file")?;
    let file = io::BufWriter::new(file);
    schema.write(file).context("Writing scenario file")?;

    Ok(())
}
