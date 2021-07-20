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
    writeln!(&mut fh, "| Mechanism | Power generation per second |")?;
    writeln!(&mut fh, "| :-: | :-: |")?;
    for reaction in def.reaction() {
        let power: units::ElectricPower = reaction
            .puts()
            .iter()
            .filter_map(|put| match put {
                reaction::Put::Electricity { base } if base.0.value() > 0.0 => Some(base.0),
                _ => None,
            })
            .sum();
        if power.0 > 0. {
            writeln!(
                &mut fh,
                "| [{}](../../reaction/{}) | {} |",
                reaction.name(),
                reaction.name().to_kebab_case(),
                power,
            )?;
        }
    }

    Ok(())
}
