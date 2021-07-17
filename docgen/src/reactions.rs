use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::KebabCase;

use super::{assets, manifest, opts};
use traffloat_types::def::{reaction, GameDefinition};

pub fn gen_reactions(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    relativize: impl Fn(&Path) -> Result<PathBuf>,
    def: &GameDefinition,
) -> Result<Vec<manifest::Nav>> {
    let mut reactions_index = vec![manifest::Nav::Path(PathBuf::from("reactions.md"))];

    for reaction in def.reaction() {
        let path = write_reaction(opts, assets, reaction, def)
            .with_context(|| format!("Writing reaction {}", reaction.name()))?;
        reactions_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    {
        let mut fh = fs::File::create(opts.root_dir.join("docs/reactions.md"))
            .context("Could not create buildings.md")?;
        writeln!(&mut fh, "# List of mechanisms")?;

        for (category_id, category) in def.reaction_cats().iter().enumerate() {
            writeln!(
                &mut fh,
                "## [{}](../{}/)",
                category.title(),
                category.title().to_kebab_case()
            )?;
            writeln!(&mut fh, "{}", category.description())?;
            for reaction in def.reaction() {
                if reaction.category().0 == category_id {
                    writeln!(
                        &mut fh,
                        "- [{}]({})",
                        reaction.name(),
                        reaction.name().to_kebab_case()
                    )?;
                }
            }
        }
    }

    Ok(reactions_index)
}

fn write_reaction(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    reaction: &reaction::Type,
    def: &GameDefinition,
) -> Result<PathBuf> {
    let reactions_dir = opts.root_dir.join("docs/reactions");
    fs::create_dir_all(&reactions_dir).context("Could not create reactions dir")?;

    let file = reactions_dir.join(format!("{}.md", reaction.name().to_kebab_case()));
    let mut fh = fs::File::create(&file)
        .with_context(|| format!("Could not open {} for writing", file.display()))?;

    writeln!(&mut fh, "# {}", reaction.name())?;
    writeln!(&mut fh, "{}", reaction.description())?;

    if !reaction.catalysts().is_empty() {
        writeln!(&mut fh, "## Catalysts/Conditions")?;
        writeln!(&mut fh, "| Type | Minimum | Maximum | Multiplier below minimum | Multiplier at minimum | Multiplier at maximum | Multiplier above maximum |")?;
        writeln!(&mut fh, "| :-: | :-: | :-: | :-: | :-: | :-: | :-: |")?;
        for catalyst in reaction.catalysts() {
            match catalyst.range() {
                reaction::CatalystRange::Cargo { ty, levels } => {
                    write!(
                        &mut fh,
                        "| {} | {} | {} ",
                        def.cargo()
                            .get(ty.0)
                            .expect("Undefined cargo reference")
                            .name(),
                        levels.start,
                        levels.end
                    )?;
                }
                reaction::CatalystRange::Liquid { ty, levels } => {
                    write!(
                        &mut fh,
                        "| {} | {} | {} ",
                        def.liquid()
                            .get(ty.0)
                            .expect("Undefined liquid reference")
                            .name(),
                        levels.start,
                        levels.end
                    )?;
                }
                reaction::CatalystRange::Gas { ty, levels } => {
                    write!(
                        &mut fh,
                        "| {} | {} | {} ",
                        def.gas().get(ty.0).expect("Undefined gas reference").name(),
                        levels.start,
                        levels.end
                    )?;
                }
                reaction::CatalystRange::Electricity { levels } => {
                    write!(
                        &mut fh,
                        "| Electricity | {} | {} ",
                        levels.start, levels.end
                    )?;
                }
                reaction::CatalystRange::Light { levels } => {
                    write!(&mut fh, "| Light | {} | {} ", levels.start, levels.end)?;
                }
                reaction::CatalystRange::Skill { ty, levels } => {
                    write!(
                        &mut fh,
                        "| {} | {} | {} ",
                        def.skill()
                            .get(ty.0)
                            .expect("Undefined skill reference")
                            .name(),
                        levels.start,
                        levels.end
                    )?;
                }
            }

            writeln!(
                &mut fh,
                "| {}x | {}x | {}x | {}x |",
                catalyst.multipliers().underflow(),
                catalyst.multipliers().min(),
                catalyst.multipliers().max(),
                catalyst.multipliers().overflow(),
            )?;
        }
        writeln!(&mut fh)?;
    }

    let inputs = reaction.puts().iter().filter(|put| put.base() < 0.);
    if inputs.clone().next().is_some() {
        writeln!(&mut fh, "## Inputs")?;
        writeln!(&mut fh, "Base consumption per second:")?;
        writeln!(&mut fh)?;
        for input in inputs {
            match input {
                reaction::Put::Cargo { ty, base } => {
                    writeln!(
                        &mut fh,
                        "- {} {}",
                        base.0 * -1.,
                        def.cargo()
                            .get(ty.0)
                            .expect("Undefined cargo reference")
                            .name()
                    )?;
                }
                reaction::Put::Liquid { ty, base } => {
                    writeln!(
                        &mut fh,
                        "- {} {}",
                        base.0 * -1.,
                        def.liquid()
                            .get(ty.0)
                            .expect("Undefined liquid reference")
                            .name()
                    )?;
                }
                reaction::Put::Gas { ty, base } => {
                    writeln!(
                        &mut fh,
                        "- {} {}",
                        base.0 * -1.,
                        def.gas().get(ty.0).expect("Undefined gas reference").name()
                    )?;
                }
                reaction::Put::Electricity { base } => {
                    writeln!(&mut fh, "- {} electricity", base.0 * -1.)?;
                }
            }
        }
        writeln!(&mut fh)?;
    }

    let outputs = reaction.puts().iter().filter(|put| put.base() > 0.);
    if outputs.clone().next().is_some() {
        writeln!(&mut fh, "## Outputs")?;
        writeln!(&mut fh, "Base production per second:")?;
        writeln!(&mut fh)?;
        for output in outputs {
            match output {
                reaction::Put::Cargo { ty, base } => {
                    writeln!(
                        &mut fh,
                        "- {} {}",
                        base.0,
                        def.cargo()
                            .get(ty.0)
                            .expect("Undefined cargo reference")
                            .name()
                    )?;
                }
                reaction::Put::Liquid { ty, base } => {
                    writeln!(
                        &mut fh,
                        "- {} {}",
                        base.0,
                        def.liquid()
                            .get(ty.0)
                            .expect("Undefined liquid reference")
                            .name()
                    )?;
                }
                reaction::Put::Gas { ty, base } => {
                    writeln!(
                        &mut fh,
                        "- {} {}",
                        base.0,
                        def.gas().get(ty.0).expect("Undefined gas reference").name()
                    )?;
                }
                reaction::Put::Electricity { base } => {
                    writeln!(&mut fh, "- {} electricity", base.0)?;
                }
            }
        }
        writeln!(&mut fh)?;
    }

    Ok(file)
}
