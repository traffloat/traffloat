use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::AtomicU32;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::{fs, io};

use anyhow::{Context as _, Result};
use codegen::{Definition, ResolveContext};
use def::atlas::{AtlasContext, IconIndex, ModelIndex};
use def::Schema;
use structopt::StructOpt;
use traffloat_def::{self as def, Def};

mod atlas;
mod lang;
mod schema;

#[derive(StructOpt)]
#[structopt(name = env!("CARGO_PKG_NAME"))]
#[structopt(version = env!("CARGO_PKG_VERSION"))]
#[structopt(author = env!("CARGO_PKG_AUTHORS"))]
#[structopt(about = env!("CARGO_PKG_DESCRIPTION"))]
struct Args {
    /// The input file
    input:    PathBuf,
    /// The output file
    output:   PathBuf,
    /// Skip SVG rendering and exporting.
    /// This is only useful for fast debugging cycles.
    /// The scenario generated will be invalid.
    #[structopt(long)]
    skip_svg: bool,
}

struct Timer {
    task:  String,
    total: Mutex<Duration>,
}

impl Timer {
    pub fn new(task: impl Into<String>) -> Self {
        Self { task: task.into(), total: Mutex::default() }
    }

    pub fn start(&self) -> TimerStart<'_> {
        TimerStart { start: Instant::now(), timer: self, report: false }
    }

    pub fn report(&self) {
        let duration = {
            let lock = self.total.lock().expect("Poisoned lock");
            Duration::clone(&lock)
        };
        log::info!("Finished {}, spent {:?}", &self.task, duration);
    }

    pub fn start_and_report(&self) -> TimerStart<'_> {
        TimerStart { start: Instant::now(), timer: self, report: true }
    }
}

struct TimerStart<'t> {
    start:  Instant,
    timer:  &'t Timer,
    report: bool,
}

impl<'t> Drop for TimerStart<'t> {
    fn drop(&mut self) {
        {
            let mut duration = self.timer.total.lock().expect("Thread panicked");
            *duration += Instant::now() - self.start;
        }
        if self.report {
            self.timer.report();
        }
    }
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = Args::from_args();

    let mut defs = Vec::new();
    let mut context = ResolveContext::new(PathBuf::new()); // path to be initialized later

    let render_timer = Rc::new(Timer::new("rendering textures"));
    let downscale_timer = Rc::new(Timer::new("downscaling textures"));
    let save_timer = Rc::new(Timer::new("saving textures"));
    let lang_parse_timer = Rc::new(Timer::new("parsing translation files"));

    fs::create_dir(&args.output).context("Creating output directory")?;

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

        {
            let mut context = context.get_other::<AtlasContext>();

            context.creation_hook = Some(Rc::new({
                let input = args.input.clone();
                let output = args.output.join("assets");
                let skip_svg = args.skip_svg;
                fs::create_dir(&output).context("Creating assets dir")?;

                let render_timer = Rc::clone(&render_timer);
                let downscale_timer = Rc::clone(&downscale_timer);
                let save_timer = Rc::clone(&save_timer);

                let next_texture_id = AtomicU32::new(0);

                move |atlas, context| {
                    let context = RefCell::new(context);
                    let args = atlas::GenerateArgs::builder()
                        .input(&input)
                        .output(&output)
                        .render_timer(&render_timer)
                        .downscale_timer(&downscale_timer)
                        .save_timer(&save_timer)
                        .atlas(atlas)
                        .next_texture_id(&next_texture_id)
                        .skip_svg(skip_svg)
                        .register_icon_texture_id(|name, id| {
                            let mut context = context.borrow_mut();
                            let mut index = context.get_other::<IconIndex>();
                            index.add(atlas.id(), name.clone(), id);
                        })
                        .register_model_texture_id(|name, id, shape| {
                            let mut context = context.borrow_mut();
                            let mut index = context.get_other::<ModelIndex>();
                            index.add(atlas.id(), name.clone(), id, shape);
                        })
                        .build();
                    atlas::generate(args)
                        .with_context(|| format!("Generating atlas from {}", atlas.dir().display()))
                }
            }))
        }

        lang::setup_context(&mut context, &lang_parse_timer);
    }

    log::info!("Loading scenario definition");
    let schema::Main { scenario, config } = {
        let timer = Timer::new("loading scenario definition");
        let _timer = timer.start_and_report();
        read_main_defs(&mut defs, &mut context, &args.input).context("Parsing input files")?
    };

    render_timer.report();
    downscale_timer.report();
    save_timer.report();
    lang_parse_timer.report();

    log::info!("Saving scenario output");
    let schema = Schema::builder().scenario(scenario).config(config).def(defs).build();
    write(&args.output.join("scenario.tfsave"), &schema).context("Saving output")?;
    lang::save(&args.output.join("assets"), &mut context)
        .context("Saving processed translations")?;

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
    let path = path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize {}", path.display()))?;
    let dir = path.parent().expect("Regular file has no parent");

    for include in file.include {
        let mut included = dir.join(&include.file);
        included = included.canonicalize().with_context(|| {
            format!("Failed to canonicalize included file {}", included.display())
        })?;
        read_included_defs(defs, context, &included)
            .with_context(|| format!("Included in {}", path.display()))?;
    }

    log::debug!("Loading {}", path.display());

    context.set_current_dir(dir.to_path_buf());
    for def in file.def {
        let def = Def::convert(def, context)
            .with_context(|| format!("Resolving references in {}", path.display()))?;
        defs.push(def);
    }

    Ok(())
}

fn write(scenario: &Path, schema: &Schema) -> Result<()> {
    let file = fs::File::create(&scenario).context("Writing scenario file")?;
    let file = io::BufWriter::new(file);
    schema.write(file).context("Writing scenario file")?;

    Ok(())
}
