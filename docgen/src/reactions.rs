use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::KebabCase;
use strum::IntoEnumIterator;

use super::{assets, manifest, opts};
use traffloat_vanilla::reactions;

pub fn gen_reactions(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    relativize: impl Fn(&Path) -> Result<PathBuf>,
) -> Result<Vec<manifest::Nav>> {
    let mut reactions_index = vec![manifest::Nav::Path(PathBuf::from("reactions.md"))];

    for reaction in &**reactions::ALL {
        let path = write_reaction(opts, assets, reaction)
            .with_context(|| format!("Writing reaction {}", reaction.name()))?;
        reactions_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    {
        let mut fh = fs::File::create(opts.root_dir.join("docs/reactions.md"))
            .context("Could not create buildings.md")?;
        writeln!(&mut fh, "# List of mechanisms")?;

        for category in reactions::Category::iter() {
            writeln!(
                &mut fh,
                "## [{}](../{}/)",
                category,
                category.to_string().to_kebab_case()
            )?;
            for reaction in &**reactions::ALL {
                if reaction.category() == category {
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
    reaction: &reactions::Def,
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
            let levels = catalyst.levels();
            writeln!(
                &mut fh,
                "| {} | {} | {} | {}x | {}x | {}x | {}x |",
                levels.ty(),
                reactions::DisplayCatalystLevel(levels, reactions::MinMax::Min),
                reactions::DisplayCatalystLevel(levels, reactions::MinMax::Max),
                catalyst.multipliers()[0],
                catalyst.multipliers()[1],
                catalyst.multipliers()[2],
                catalyst.multipliers()[3],
            )?;
        }
        writeln!(&mut fh)?;
    }

    let inputs = reaction
        .puts()
        .iter()
        .filter(|put| put.rate().0.size() < 0.);
    if inputs.clone().next().is_some() {
        writeln!(&mut fh, "## Inputs")?;
        writeln!(&mut fh, "Base consumption per second:")?;
        writeln!(&mut fh)?;
        for input in inputs {
            let mut consume = input.rate().0.clone();
            *consume.size_mut() *= -1.;
            writeln!(&mut fh, "- {}", &input.rate().0)?;
        }
        writeln!(&mut fh)?;
    }

    let outputs = reaction
        .puts()
        .iter()
        .filter(|put| put.rate().0.size() > 0.);
    if outputs.clone().next().is_some() {
        writeln!(&mut fh, "## Outputs")?;
        writeln!(&mut fh, "Base production per second:")?;
        writeln!(&mut fh)?;
        for output in outputs {
            writeln!(&mut fh, "- {}", &output.rate().0)?;
        }
        writeln!(&mut fh)?;
    }

    Ok(file)
}
