use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::KebabCase;

use super::{assets, opts};
use traffloat_types::def::{reaction, GameDefinition};
use traffloat_types::units;

pub fn gen_electricity(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    relativize: impl Fn(&Path) -> Result<PathBuf>,
    def: &GameDefinition,
) -> Result<()> {
    let mut fh = fs::File::create(opts.root_dir.join("docs/electricity.md"))
        .context("Could not create electricity.md")?;
    writeln!(&mut fh, "{}", include_str!("electricity.md"))?;

    writeln!(&mut fh, "## Electricity generation")?;
    writeln!(
        &mut fh,
        "| Mechanism | Power generation per second | Buildings |"
    )?;
    writeln!(&mut fh, "| :-: | :-: | :-: |")?;
    for (reaction_id, reaction) in def.reaction().iter().enumerate() {
        let power: units::ElectricPower = reaction
            .puts()
            .iter()
            .filter_map(|put| match put {
                reaction::Put::Electricity { base } if base.0.value() > 0.0 => Some(base.0),
                _ => None,
            })
            .sum();
        if power.0 > 0. {
            let buildings = def
                .building()
                .iter()
                .filter(|building| {
                    building
                        .reactions()
                        .iter()
                        .any(|(id, _)| id.0 == reaction_id)
                })
                .map(|building| {
                    let texture_dir = opts
                        .client_dir
                        .join("textures")
                        .join(building.shape().texture_name().as_str());
                    let texture_dir = texture_dir.canonicalize().with_context(|| {
                        format!("Could not canonicalize {}", texture_dir.display())
                    })?;
                    Ok(format!(
                        "[![]({})](../building/{})",
                        assets.map(&texture_dir.join("xp.png"))?,
                        building.name().to_kebab_case(),
                    ))
                })
                .collect::<Result<Vec<String>>>()?;

            writeln!(
                &mut fh,
                "| {} | {} | {} |",
                reaction.name(),
                power,
                buildings.join(" "),
            )?;
        }
    }

    Ok(())
}
