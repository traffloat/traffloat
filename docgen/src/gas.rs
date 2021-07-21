use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::KebabCase;

use super::{assets, manifest, opts};
use traffloat_types::def::{gas, reaction, GameDefinition};

pub fn gen_gases(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    relativize: impl Fn(&Path) -> Result<PathBuf>,
    def: &GameDefinition,
) -> Result<Vec<manifest::Nav>> {
    let mut gases_index = vec![manifest::Nav::Path(PathBuf::from("gas.md"))];

    for (gas_id, gas) in def.gas().iter().enumerate() {
        let path = write_gas(opts, assets, gas_id, gas, def)
            .with_context(|| format!("Writing gas {}", gas.name()))?;
        gases_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    {
        let mut fh = fs::File::create(opts.root_dir.join("docs/gas.md"))
            .context("Could not create gas.md")?;
        writeln!(&mut fh, "{}", include_str!("gas.md"))?;
        writeln!(&mut fh, "# List of gas types")?;

        for gas in def.gas() {
            let texture_path = opts
                .client_dir
                .join("textures")
                .join(gas.texture().as_str())
                .with_extension("png");
            let texture_path = texture_path
                .canonicalize()
                .with_context(|| format!("Could not canonicalize {}", texture_path.display()))?;
            writeln!(
                &mut fh,
                "- [![]({}){{ width=24 }} {}]({})",
                assets.map(&texture_path)?,
                gas.name(),
                gas.name().to_kebab_case()
            )?;
        }
    }

    Ok(gases_index)
}

fn write_gas(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    gas_id: usize,
    gas: &gas::Type,
    def: &GameDefinition,
) -> Result<PathBuf> {
    let gases_dir = opts.root_dir.join("docs/gas");
    fs::create_dir_all(&gases_dir).context("Could not create gas dir")?;

    let file = gases_dir.join(format!("{}.md", gas.name().to_kebab_case()));
    let mut fh = fs::File::create(&file)
        .with_context(|| format!("Could not open {} for writing", file.display()))?;

    let texture_path = opts
        .client_dir
        .join("textures")
        .join(gas.texture().as_str())
        .with_extension("png");
    let texture_path = texture_path
        .canonicalize()
        .with_context(|| format!("Could not canonicalize {}", texture_path.display()))?;

    writeln!(&mut fh, "# {}", gas.name())?;
    writeln!(
        &mut fh,
        "![](../{}){{ width=64 align=right }}",
        assets.map(&texture_path)?
    )?;
    writeln!(&mut fh, "> {}", gas.summary())?;
    writeln!(&mut fh)?;
    writeln!(&mut fh, "{}", gas.description())?;

    fn is_gas_range(range: &reaction::CatalystRange, gas_id: usize) -> bool {
        match range {
            reaction::CatalystRange::Gas { ty, .. } => ty.0 == gas_id,
            _ => false,
        }
    }
    fn is_gas_put(put: &reaction::Put, gas_id: usize) -> bool {
        match put {
            reaction::Put::Gas { ty, .. } => ty.0 == gas_id,
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
                .any(|catalyst| is_gas_range(catalyst.range(), gas_id))
        })
        .collect();
    let as_input = def
        .reaction()
        .iter()
        .filter(|reaction| {
            reaction
                .puts()
                .iter()
                .any(|put| is_gas_put(put, gas_id) && put.is_input())
        })
        .collect();
    let as_output = def
        .reaction()
        .iter()
        .filter(|reaction| {
            reaction
                .puts()
                .iter()
                .any(|put| is_gas_put(put, gas_id) && put.is_output())
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
