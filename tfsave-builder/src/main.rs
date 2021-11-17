use std::any::TypeId;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::AtomicU32;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::{fs, io};

use anyhow::{Context as _, Result};
use def::atlas::xy::{AtlasContext, AtlasCreationHook, IconIndex, ModelIndex};
use def::curdir::CurrentDir;
use def::{Schema, TfsaveFile};
use structopt::StructOpt;
use traffloat_def::{self as def, AnyDef};
use xylem::{Context as _, DefaultContext, NoArgs};

mod atlas;
mod init;
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
    let mut init = Vec::new();
    let mut context = DefaultContext::default();

    let render_timer = Rc::new(Timer::new("rendering textures"));
    let downscale_timer = Rc::new(Timer::new("downscaling textures"));
    let save_timer = Rc::new(Timer::new("saving textures"));
    let lang_parse_timer = Rc::new(Timer::new("parsing translation files"));

    fs::create_dir(&args.output).context("Creating output directory")?;

    {
        {
            let creation_hook: Rc<AtlasCreationHook> = Rc::new({
                let input = args.input.clone();
                let output = args.output.clone();
                let skip_svg = args.skip_svg;
                fs::create_dir(output.join("assets")).context("Creating assets dir")?;

                let render_timer = Rc::clone(&render_timer);
                let downscale_timer = Rc::clone(&downscale_timer);
                let save_timer = Rc::clone(&save_timer);

                let next_texture_id = AtomicU32::new(0);

                move |atlas, context| {
                    let args = atlas::GenerateArgs::builder()
                        .input(&input)
                        .output(&output)
                        .render_timer(&render_timer)
                        .downscale_timer(&downscale_timer)
                        .save_timer(&save_timer)
                        .atlas(atlas)
                        .next_texture_id(&next_texture_id)
                        .skip_svg(skip_svg)
                        .register_icon_texture_id(|context, name, id| {
                            let index = context
                                .get_mut::<IconIndex, _>(TypeId::of::<()>(), Default::default);
                            index.add(atlas.id(), name.to_string(), id);
                        })
                        .register_model_texture_id(|context, name, id, shape| {
                            let index = context
                                .get_mut::<ModelIndex, _>(TypeId::of::<()>(), Default::default);
                            index.add(atlas.id(), name.to_string(), id, shape);
                        })
                        .build();
                    atlas::generate(context, args)
                        .with_context(|| format!("Generating atlas from {}", atlas.dir().display()))
                }
            });

            context.get_mut::<AtlasContext, _>(TypeId::of::<()>(), move || AtlasContext {
                creation_hook,
            });
        }

        lang::setup_context(&mut context, &lang_parse_timer);
    }

    log::info!("Loading scenario definition");
    let schema::Main { scenario, config, state: scalar_state } = {
        let timer = Timer::new("loading scenario definition");
        let _timer = timer.start_and_report();
        read_main_defs(&mut defs, &mut init, &mut context, &args.input)
            .context("Parsing input files")?
    };

    render_timer.report();
    downscale_timer.report();
    save_timer.report();
    lang_parse_timer.report();

    let state = init::resolve_states(init, &scalar_state);

    log::info!("Saving scenario output");
    let schema =
        TfsaveFile::builder().scenario(scenario).config(config).def(defs).state(state).build();
    write(&args.output.join("scenario.tfsave"), &schema).context("Saving output")?;
    lang::save(&args.output.join("assets"), &mut context)
        .context("Saving processed translations")?;

    Ok(())
}

fn read_main_defs(
    defs: &mut Vec<AnyDef>,
    inits: &mut Vec<init::Init>,
    context: &mut DefaultContext,
    path: &Path,
) -> Result<schema::Main> {
    let string = fs::read_to_string(path).with_context(|| format!("Reading {}", path.display()))?;
    let mut de = toml::Deserializer::new(&string);
    let toml: schema::MainFile = serde_path_to_error::deserialize(&mut de)
        .with_context(|| format!("Parsing {}", path.display()))?;

    read_defs(defs, inits, context, toml.file, path)?;

    Ok(toml.main)
}

fn read_included_defs(
    defs: &mut Vec<AnyDef>,
    inits: &mut Vec<init::Init>,
    context: &mut DefaultContext,
    path: &Path,
) -> Result<()> {
    let string = fs::read_to_string(path).with_context(|| format!("Reading {}", path.display()))?;
    let mut de = toml::Deserializer::new(&string);
    let toml: schema::File = serde_path_to_error::deserialize(&mut de)
        .with_context(|| format!("Parsing {}", path.display()))?;

    read_defs(defs, inits, context, toml, path)
}

fn read_defs(
    defs: &mut Vec<AnyDef>,
    inits: &mut Vec<init::Init>,
    context: &mut DefaultContext,
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
        read_included_defs(defs, inits, context, &included)
            .with_context(|| format!("Included in {}", path.display()))?;
    }

    log::debug!("Loading {}", path.display());

    context
        .get_mut::<CurrentDir, _>(TypeId::of::<()>(), move || CurrentDir::new(PathBuf::new()))
        .set_path(dir.to_path_buf());

    for def in file.def {
        let def = <AnyDef as xylem::Xylem<Schema>>::convert(def, context, &NoArgs)
            .with_context(|| format!("Resolving references in {}", path.display()))?;
        defs.push(def);
    }

    for init in file.init {
        let init = <init::Init as xylem::Xylem<Schema>>::convert(init, context, &NoArgs)
            .with_context(|| format!("Resolving references in {}", path.display()))?;
        inits.push(init);
    }

    Ok(())
}

fn write(scenario: &Path, schema: &TfsaveFile) -> Result<()> {
    let file = fs::File::create(&scenario).context("Writing scenario file")?;
    let file = io::BufWriter::new(file);
    schema.write(file).context("Writing scenario file")?;

    Ok(())
}
