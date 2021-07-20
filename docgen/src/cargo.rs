use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::KebabCase;

use super::{assets, manifest, opts};
use traffloat_types::def::{cargo, reaction, GameDefinition};

pub fn gen_cargos(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    relativize: impl Fn(&Path) -> Result<PathBuf>,
    def: &GameDefinition,
) -> Result<Vec<manifest::Nav>> {
    let mut cargos_index = vec![manifest::Nav::Path(PathBuf::from("cargo.md"))];

    for (cargo_id, cargo) in def.cargo().iter().enumerate() {
        let path = write_cargo(opts, assets, cargo_id, cargo, def)
            .with_context(|| format!("Writing cargo {}", cargo.name()))?;
        cargos_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    {
        let mut fh = fs::File::create(opts.root_dir.join("docs/cargo.md"))
            .context("Could not create cargo.md")?;
        writeln!(&mut fh, "{}", include_str!("cargo.md"))?;
        writeln!(&mut fh, "## List of cargo types")?;

        for (category_id, category) in def.cargo_cats().iter().enumerate() {
            writeln!(&mut fh, "### {}", category.title())?;
            writeln!(&mut fh, "{}", category.description())?;
            writeln!(&mut fh)?;

            for cargo in def.cargo() {
                if cargo.category().0 == category_id {
                    let texture_path = opts
                        .client_dir
                        .join("textures")
                        .join(cargo.texture())
                        .with_extension("png");
                    let texture_path = texture_path.canonicalize().with_context(|| {
                        format!("Could not canonicalize {}", texture_path.display())
                    })?;
                    writeln!(
                        &mut fh,
                        "- [![]({}){{ width=24 }} {}]({})",
                        assets.map(&texture_path)?,
                        cargo.name(),
                        cargo.name().to_kebab_case()
                    )?;
                }
            }
        }
    }

    Ok(cargos_index)
}

fn write_cargo(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    cargo_id: usize,
    cargo: &cargo::Type,
    def: &GameDefinition,
) -> Result<PathBuf> {
    let cargos_dir = opts.root_dir.join("docs/cargo");
    fs::create_dir_all(&cargos_dir).context("Could not create cargo dir")?;

    let file = cargos_dir.join(format!("{}.md", cargo.name().to_kebab_case()));
    let mut fh = fs::File::create(&file)
        .with_context(|| format!("Could not open {} for writing", file.display()))?;

    let texture_path = opts
        .client_dir
        .join("textures")
        .join(cargo.texture())
        .with_extension("png");
    let texture_path = texture_path
        .canonicalize()
        .with_context(|| format!("Could not canonicalize {}", texture_path.display()))?;

    writeln!(&mut fh, "# {}", cargo.name())?;
    writeln!(
        &mut fh,
        "![](../{}){{ width=64 align=right }}",
        assets.map(&texture_path)?
    )?;
    writeln!(&mut fh, "> {}", cargo.summary())?;
    writeln!(&mut fh)?;
    writeln!(&mut fh, "{}", cargo.description())?;

    fn is_cargo_range(range: &reaction::CatalystRange, cargo_id: usize) -> bool {
        match range {
            reaction::CatalystRange::Cargo { ty, .. } => ty.0 == cargo_id,
            _ => false,
        }
    }
    fn is_cargo_put(put: &reaction::Put, cargo_id: usize) -> bool {
        match put {
            reaction::Put::Cargo { ty, .. } => ty.0 == cargo_id,
            _ => false,
        }
    }

    let as_catalyst = def
        .reaction()
        .iter()
        .filter(|reaction| {
            reaction
                .catalysts()
                .iter()
                .any(|catalyst| is_cargo_range(catalyst.range(), cargo_id))
        })
        .collect();
    let as_input = def
        .reaction()
        .iter()
        .filter(|reaction| {
            reaction
                .puts()
                .iter()
                .any(|put| is_cargo_put(put, cargo_id) && put.is_input())
        })
        .collect();
    let as_output = def
        .reaction()
        .iter()
        .filter(|reaction| {
            reaction
                .puts()
                .iter()
                .any(|put| is_cargo_put(put, cargo_id) && put.is_output())
        })
        .collect();
    let reaction_groups: [(&str, Vec<_>); 3] = [
        ("Produced by", as_output),
        ("Consumed by", as_input),
        ("Catalyzes", as_catalyst),
    ];

    for (title, group) in &reaction_groups {
        if !group.is_empty() {
            writeln!(&mut fh, "## {}", title)?;
            for reaction in group {
                writeln!(&mut fh, "- {}", reaction.name())?;
            }
            writeln!(&mut fh)?;
        }
    }

    Ok(file)
}
