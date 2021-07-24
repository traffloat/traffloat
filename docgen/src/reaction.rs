use std::io::Write;

use anyhow::Result;
use heck::KebabCase;

use super::{assets, opts};
use traffloat_types::def::{reaction, GameDefinition};

pub fn write_reaction(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    reaction: &reaction::Type,
    def: &GameDefinition,
    mut fh: impl Write,
    nesting: &str,
) -> Result<()> {
    writeln!(&mut fh, "{} {}", nesting, reaction.name())?;
    writeln!(&mut fh, "{}", reaction.description())?;

    if !reaction.catalysts().is_empty() {
        writeln!(&mut fh, "{}# Catalysts/Conditions", nesting)?;
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

    for (positive, title, noun, th, mul) in [
        (false, "Inputs", "consumption", "Input type", -1.),
        (true, "Outputs", "production", "Output type", 1.),
    ] {
        let puts = reaction
            .puts()
            .iter()
            .filter(|put| (put.is_output()) == positive);
        if puts.clone().next().is_some() {
            writeln!(&mut fh, "{}# {}", nesting, title)?;
            writeln!(&mut fh, "| {} | Base {} per second |", th, noun)?;
            writeln!(&mut fh, "| :-: | :-: |")?;
            for put in puts {
                match put {
                    reaction::Put::Cargo { ty, base } => {
                        writeln!(
                            &mut fh,
                            "| [{}](../../cargo/{}) | {} |",
                            def.get_cargo(*ty).name(),
                            def.get_cargo(*ty).name().to_kebab_case(),
                            base.0 * mul,
                        )?;
                    }
                    reaction::Put::Liquid { ty, base } => {
                        writeln!(
                            &mut fh,
                            "| [{}](../../cargo/{}) | {} |",
                            def.get_liquid(*ty).name(),
                            def.get_liquid(*ty).name().to_kebab_case(),
                            base.0 * mul,
                        )?;
                    }
                    reaction::Put::Gas { ty, base } => {
                        writeln!(
                            &mut fh,
                            "| [{}](../../cargo/{}) | {} |",
                            def.get_gas(*ty).name(),
                            def.get_gas(*ty).name().to_kebab_case(),
                            base.0 * mul,
                        )?;
                    }
                    reaction::Put::Electricity { base } => {
                        writeln!(
                            &mut fh,
                            "| [Electricity](../../electricity) | {} |",
                            base.0 * mul,
                        )?;
                    }
                    reaction::Put::Skill { ty, base } => {
                        writeln!(
                            &mut fh,
                            "| [{}](../../skill/{}) | {} |",
                            def.get_skill(*ty).name(),
                            def.get_skill(*ty).name().to_kebab_case(),
                            base.0 * mul,
                        )?;
                    }
                }
            }
            writeln!(&mut fh)?;
        }
    }

    Ok(())
}
