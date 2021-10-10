use std::any::TypeId;
use std::cell::RefCell;
use std::collections::{btree_map, BTreeMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{Context as _, Result};
use arcstr::ArcStr;
use codegen::{Definition, Identifiable, ResolveContext};
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

    let defs = Rc::new(DefList::default());
    let schema::Main { scenario, config } =
        read_main_defs(&defs, &args.input).context("Parsing input files")?;

    Ok(())
}

#[derive(Default)]
struct DefList {
    vec:   RefCell<Vec<Def>>,
    index: RefCell<BTreeMap<TypeId, BTreeMap<ArcStr, usize>>>,
}

impl DefList {
    pub fn pre_push(&self, id_str: ArcStr, def: &def::DefHumanFriendly) -> Result<()> {
        let type_id = match def.value_type_id() {
            Some(id) => id,
            _ => return Ok(()), // no need to track
        };
        let type_name = def.value_type_name();

        {
            let mut index = self.index.borrow_mut();
            let index = index.entry(type_id).or_default();
            let ord = index.len();

            match index.entry(id_str.clone()) {
                btree_map::Entry::Vacant(entry) => entry.insert(ord),
                btree_map::Entry::Occupied(_) => anyhow::bail!(
                    "Duplicate {} definition {}",
                    type_name.expect("type_id is Some"),
                    id_str
                ),
            };
        }

        Ok(())
    }

    pub fn push(&self, def: Def) {
        let mut vec = self.vec.borrow_mut();
        vec.push(def);
    }

    pub fn add_context<T: Identifiable + 'static>(self: &Rc<Self>, context: &mut ResolveContext) {
        let this = Rc::clone(self);
        context.add::<T>(Box::new(move |name| {
            let index = this.index.borrow();
            let subindex = index.get(&TypeId::of::<T>())?;
            let id = subindex.get(name)?;
            Some(*id)
        }));
    }
}

fn read_main_defs(defs: &Rc<DefList>, path: &Path) -> Result<schema::Main> {
    let string = fs::read_to_string(path).with_context(|| format!("Reading {}", path.display()))?;
    let mut de = toml::Deserializer::new(&string);
    let toml: schema::MainFile = serde_path_to_error::deserialize(&mut de)
        .with_context(|| format!("Parsing {}", path.display()))?;

    read_defs(defs, toml.file, path)?;

    Ok(toml.main)
}

fn read_included_defs(defs: &Rc<DefList>, path: &Path) -> Result<()> {
    let string = fs::read_to_string(path).with_context(|| format!("Reading {}", path.display()))?;
    let mut de = toml::Deserializer::new(&string);
    let toml: schema::File = serde_path_to_error::deserialize(&mut de)
        .with_context(|| format!("Parsing {}", path.display()))?;

    read_defs(defs, toml, path)
}

fn read_defs(defs: &Rc<DefList>, file: schema::File, path: &Path) -> Result<()> {
    for include in file.include {
        let path = path
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize {}", path.display()))?;
        let dir = path.parent().context("Regular file has no parent")?;

        let mut included = dir.join(&include.file);
        included = included.canonicalize().with_context(|| {
            format!("Failed to canonicalize included file {}", included.display())
        })?;
        read_included_defs(defs, &included)
            .with_context(|| format!("Included in {}", path.display()))?;
    }

    log::info!("Loading {}", path.display());
    let mut context = ResolveContext::default();

    defs.add_context::<def::lang::Def>(&mut context);
    defs.add_context::<def::atlas::Def>(&mut context);
    defs.add_context::<def::liquid::Def>(&mut context);
    defs.add_context::<def::gas::Def>(&mut context);
    defs.add_context::<def::cargo::category::Def>(&mut context);
    defs.add_context::<def::cargo::Def>(&mut context);
    defs.add_context::<def::skill::Def>(&mut context);
    defs.add_context::<def::vehicle::Def>(&mut context);
    defs.add_context::<def::building::category::Def>(&mut context);
    defs.add_context::<def::building::Def>(&mut context);
    defs.add_context::<def::crime::Def>(&mut context);

    for def in file.def {
        if let Some(id_str) = def.id_str() {
            let id_str = id_str.clone();
            defs.pre_push(id_str, &def).with_context(|| format!("Loading {}", path.display()))?;
            let def = Def::convert(def, &context)
                .with_context(|| format!("Resolving references in {}", path.display()))?;
            defs.push(def);
        }
    }

    Ok(())
}
