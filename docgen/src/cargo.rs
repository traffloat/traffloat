use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::KebabCase;
use strum::IntoEnumIterator;

use super::{assets, manifest, opts};
use traffloat_vanilla::{cargo, reactions};

pub fn gen_cargos(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    relativize: impl Fn(&Path) -> Result<PathBuf>,
) -> Result<Vec<manifest::Nav>> {
    let mut cargos_index = vec![manifest::Nav::Path(PathBuf::from("cargo.md"))];

    for cargo in &**cargo::ALL {
        let path = write_cargo(opts, assets, cargo)
            .with_context(|| format!("Writing cargo {}", cargo.name))?;
        cargos_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    {
        let mut fh = fs::File::create(opts.root_dir.join("docs/cargo.md"))
            .context("Could not create cargo.md")?;
        writeln!(&mut fh, "{}", include_str!("cargo.md"))?;
        writeln!(&mut fh, "## List of cargo types")?;

        for category in cargo::Category::iter() {
            writeln!(&mut fh, "### {}", category)?;
            writeln!(&mut fh, "{}", cargo::category_description(category))?;
            writeln!(&mut fh)?;

            for cargo in &**cargo::ALL {
                if cargo.category == category {
                    let texture_path = opts
                        .client_dir
                        .join("textures")
                        .join(cargo.texture.as_ref())
                        .with_extension("png");
                    let texture_path = texture_path.canonicalize().with_context(|| {
                        format!("Could not canonicalize {}", texture_path.display())
                    })?;
                    writeln!(
                        &mut fh,
                        "- [![]({}){{ width=24 }} {}]({})",
                        assets.map(&texture_path)?,
                        cargo.name,
                        cargo.name.to_kebab_case()
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
    cargo: &cargo::Def,
) -> Result<PathBuf> {
    let cargos_dir = opts.root_dir.join("docs/cargo");
    fs::create_dir_all(&cargos_dir).context("Could not create cargo dir")?;

    let file = cargos_dir.join(format!("{}.md", cargo.name.to_kebab_case()));
    let mut fh = fs::File::create(&file)
        .with_context(|| format!("Could not open {} for writing", file.display()))?;

    let texture_path = opts
        .client_dir
        .join("textures")
        .join(cargo.texture.as_ref())
        .with_extension("png");
    let texture_path = texture_path
        .canonicalize()
        .with_context(|| format!("Could not canonicalize {}", texture_path.display()))?;

    writeln!(&mut fh, "# {}", cargo.name)?;
    writeln!(
        &mut fh,
        "![](../{}){{ width=64 align=right }}",
        assets.map(&texture_path)?
    )?;
    writeln!(&mut fh, "> {}", cargo.summary)?;
    writeln!(&mut fh)?;
    writeln!(&mut fh, "{}", cargo.description)?;

    let as_catalyst = reactions::ALL
        .iter()
        .filter(|reaction| {
            reaction
                .catalysts()
                .iter()
                .any(|catalyst| catalyst.levels().ty() == cargo.name)
        })
        .collect();
    let as_input = reactions::ALL
        .iter()
        .filter(|reaction| {
            reaction
                .puts()
                .iter()
                .any(|put| put.rate().0.ty() == cargo.name && put.rate().0.size() < 0.)
        })
        .collect();
    let as_output = reactions::ALL
        .iter()
        .filter(|reaction| {
            reaction
                .puts()
                .iter()
                .any(|put| put.rate().0.ty() == cargo.name && put.rate().0.size() > 0.)
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
