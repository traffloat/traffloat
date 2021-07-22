use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::KebabCase;

use super::{assets, manifest, opts};
use traffloat_types::def::{liquid, reaction, GameDefinition};

pub fn gen_liquids(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    relativize: impl Fn(&Path) -> Result<PathBuf>,
    def: &GameDefinition,
) -> Result<Vec<manifest::Nav>> {
    let mut liquids_index = vec![manifest::Nav::Path(PathBuf::from("liquid.md"))];

    for (liquid_id, liquid) in def.liquid().iter().enumerate() {
        let path = write_liquid(opts, assets, liquid_id, liquid, def)
            .with_context(|| format!("Writing liquid {}", liquid.name()))?;
        liquids_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    {
        let mut fh = fs::File::create(opts.root_dir.join("docs/liquid.md"))
            .context("Could not create liquid.md")?;
        writeln!(&mut fh, "{}", include_str!("liquid.md"))?;
        writeln!(&mut fh, "## List of liquid types")?;

        for liquid in def.liquid() {
            let texture_path = opts
                .client_dir
                .join("textures")
                .join(liquid.texture().as_str())
                .with_extension("svg");
            let texture_path = texture_path
                .canonicalize()
                .with_context(|| format!("Could not canonicalize {}", texture_path.display()))?;
            writeln!(
                &mut fh,
                "- [![]({}){{ width=20 }} {}]({})",
                assets.map(&texture_path)?,
                liquid.name(),
                liquid.name().to_kebab_case()
            )?;
        }
    }

    Ok(liquids_index)
}

fn write_liquid(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    liquid_id: usize,
    liquid: &liquid::Type,
    def: &GameDefinition,
) -> Result<PathBuf> {
    let liquids_dir = opts.root_dir.join("docs/liquid");
    fs::create_dir_all(&liquids_dir).context("Could not create liquid dir")?;

    let file = liquids_dir.join(format!("{}.md", liquid.name().to_kebab_case()));
    let mut fh = fs::File::create(&file)
        .with_context(|| format!("Could not open {} for writing", file.display()))?;

    let texture_path = opts
        .client_dir
        .join("textures")
        .join(liquid.texture().as_str())
        .with_extension("svg");
    let texture_path = texture_path
        .canonicalize()
        .with_context(|| format!("Could not canonicalize {}", texture_path.display()))?;

    writeln!(&mut fh, "# {}", liquid.name())?;
    writeln!(
        &mut fh,
        "![](../{}){{ width=64 align=right }}",
        assets.map(&texture_path)?
    )?;
    writeln!(&mut fh, "> {}", liquid.summary())?;
    writeln!(&mut fh)?;
    writeln!(&mut fh, "{}", liquid.description())?;
    writeln!(&mut fh)?;

    fn is_liquid_range(range: &reaction::CatalystRange, liquid_id: usize) -> bool {
        match range {
            reaction::CatalystRange::Liquid { ty, .. } => ty.0 == liquid_id,
            _ => false,
        }
    }
    fn is_liquid_put(put: &reaction::Put, liquid_id: usize) -> bool {
        match put {
            reaction::Put::Liquid { ty, .. } => ty.0 == liquid_id,
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
                .any(|catalyst| is_liquid_range(catalyst.range(), liquid_id))
        })
        .collect();
    let as_input = def
        .reaction()
        .iter()
        .filter(|reaction| {
            reaction
                .puts()
                .iter()
                .any(|put| is_liquid_put(put, liquid_id) && put.is_input())
        })
        .collect();
    let as_output = def
        .reaction()
        .iter()
        .filter(|reaction| {
            reaction
                .puts()
                .iter()
                .any(|put| is_liquid_put(put, liquid_id) && put.is_output())
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
