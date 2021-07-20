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
    let mut reactions_index = vec![manifest::Nav::Path(PathBuf::from("reaction.md"))];

    for reaction in def.reaction() {
        let path = write_reaction(opts, assets, reaction, def)
            .with_context(|| format!("Writing reaction {}", reaction.name()))?;
        reactions_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    {
        let mut fh = fs::File::create(opts.root_dir.join("docs/reaction.md"))
            .context("Could not create building.md")?;
        writeln!(&mut fh, "# List of mechanisms")?;

        for (category_id, category) in def.reaction_cats().iter().enumerate() {
            writeln!(
                &mut fh,
                "## [{}](../{}/)",
                category.title(),
                category.title().to_kebab_case()
            )?;
            writeln!(&mut fh, "{}", category.description())?;
            writeln!(&mut fh)?;
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
    let reactions_dir = opts.root_dir.join("docs/reaction");
    fs::create_dir_all(&reactions_dir).context("Could not create reaction dir")?;

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
                        "| [{}](../../cargo/{}) | {} | {} ",
                        def.get_cargo(*ty).name(),
                        def.get_cargo(*ty).name().to_kebab_case(),
                        levels.start,
                        levels.end,
                    )?;
                }
                reaction::CatalystRange::Liquid { ty, levels } => {
                    write!(
                        &mut fh,
                        "| [{}](../../liquid/{}) | {} | {} ",
                        def.get_liquid(*ty).name(),
                        def.get_liquid(*ty).name().to_kebab_case(),
                        levels.start,
                        levels.end,
                    )?;
                }
                reaction::CatalystRange::Gas { ty, levels } => {
                    write!(
                        &mut fh,
                        "| [{}](../../gas/{}) | {} | {} ",
                        def.get_gas(*ty).name(),
                        def.get_gas(*ty).name().to_kebab_case(),
                        levels.start,
                        levels.end,
                    )?;
                }
                reaction::CatalystRange::Electricity { levels } => {
                    write!(
                        &mut fh,
                        "| [Electricity](../../electricity) | {} | {} ",
                        levels.start, levels.end,
                    )?;
                }
                reaction::CatalystRange::Light { levels } => {
                    write!(
                        &mut fh,
                        "| [Light](../../sun) | {} | {} ",
                        levels.start, levels.end
                    )?;
                }
                reaction::CatalystRange::Skill { ty, levels } => {
                    write!(
                        &mut fh,
                        "| [{}](../../skill/{}) | {} | {} ",
                        def.get_skill(*ty).name(),
                        def.get_skill(*ty).name().to_kebab_case(),
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

    for (positive, title, noun, mul) in [
        (false, "Inputs", "consumption", -1.),
        (true, "Outputs", "production", 1.),
    ] {
        let puts = reaction
            .puts()
            .iter()
            .filter(|put| (put.is_output()) == positive);
        if puts.clone().next().is_some() {
            writeln!(&mut fh, "## {}", title)?;
            writeln!(&mut fh, "Base {} per second:", noun)?;
            writeln!(&mut fh)?;
            for put in puts {
                match put {
                    reaction::Put::Cargo { ty, base } => {
                        writeln!(
                            &mut fh,
                            "- {} [{}](../../cargo/{})",
                            base.0 * mul,
                            def.get_cargo(*ty).name(),
                            def.get_cargo(*ty).name().to_kebab_case(),
                        )?;
                    }
                    reaction::Put::Liquid { ty, base } => {
                        writeln!(
                            &mut fh,
                            "- {} [{}](../../cargo/{})",
                            base.0 * mul,
                            def.get_liquid(*ty).name(),
                            def.get_liquid(*ty).name().to_kebab_case(),
                        )?;
                    }
                    reaction::Put::Gas { ty, base } => {
                        writeln!(
                            &mut fh,
                            "- {} [{}](../../cargo/{})",
                            base.0 * mul,
                            def.get_gas(*ty).name(),
                            def.get_gas(*ty).name().to_kebab_case(),
                        )?;
                    }
                    reaction::Put::Electricity { base } => {
                        writeln!(
                            &mut fh,
                            "- {} [electricity](../../electricity)",
                            base.0 * mul,
                        )?;
                    }
                    reaction::Put::Happiness { base } => {
                        writeln!(
                            &mut fh,
                            "- {:+} [electricity](../../electricity)",
                            base.0 * mul,
                        )?;
                    }
                }
            }
            writeln!(&mut fh)?;
        }
    }

    Ok(file)
}
