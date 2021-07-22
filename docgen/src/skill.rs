use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::KebabCase;

use super::{assets, manifest, opts};
use traffloat_types::def::{reaction, skill, GameDefinition};

pub fn gen_skills(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    relativize: impl Fn(&Path) -> Result<PathBuf>,
    def: &GameDefinition,
) -> Result<Vec<manifest::Nav>> {
    let mut skills_index = vec![manifest::Nav::Path(PathBuf::from("skill.md"))];

    for (skill_id, skill) in def.skill().iter().enumerate() {
        let path = write_skill(opts, assets, skill_id, skill, def)
            .with_context(|| format!("Writing skill {}", skill.name()))?;
        skills_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    {
        let mut fh = fs::File::create(opts.root_dir.join("docs/skill.md"))
            .context("Could not create skill.md")?;
        writeln!(&mut fh, "{}", include_str!("skill.md"))?;
        writeln!(&mut fh, "## List of skill types")?;

        for skill in def.skill() {
            writeln!(
                &mut fh,
                "- [{}]({})",
                skill.name(),
                skill.name().to_kebab_case()
            )?;
        }
    }

    Ok(skills_index)
}

fn write_skill(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    skill_id: usize,
    skill: &skill::Type,
    def: &GameDefinition,
) -> Result<PathBuf> {
    let skills_dir = opts.root_dir.join("docs/skill");
    fs::create_dir_all(&skills_dir).context("Could not create skill dir")?;

    let file = skills_dir.join(format!("{}.md", skill.name().to_kebab_case()));
    let mut fh = fs::File::create(&file)
        .with_context(|| format!("Could not open {} for writing", file.display()))?;

    writeln!(&mut fh, "# {}", skill.name())?;
    writeln!(&mut fh, "{}", skill.description())?;
    writeln!(&mut fh)?;

    let reactions = def
        .reaction()
        .iter()
        .enumerate()
        .filter_map(|(reaction_id, reaction)| {
            let catalyst =
                reaction
                    .catalysts()
                    .iter()
                    .find_map(|catalyst| match catalyst.range() {
                        reaction::CatalystRange::Skill { ty, levels } if ty.0 == skill_id => {
                            Some(levels)
                        }
                        _ => None,
                    })?;
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
                        "[![](../{}){{ width=20 }}](../../building/{})",
                        assets.map(&texture_dir.join("xp.svg"))?,
                        building.name().to_kebab_case(),
                    ))
                })
                .collect::<Result<Vec<String>>>();
            let buildings = match buildings {
                Ok(buildings) => buildings,
                Err(err) => return Some(Err(err)),
            };
            Some(Ok(format!(
                "| {} | {} | {} | {} |",
                reaction.name(),
                catalyst.start,
                catalyst.end,
                buildings.join(" "),
            )))
        })
        .collect::<Result<Vec<_>>>()?;

    if !reactions.is_empty() {
        writeln!(&mut fh, "## Mechanisms")?;
        writeln!(
            &mut fh,
            "| Mechanism | Minimum {0} | Maximum {0} | Buildings |",
            skill.name(),
        )?;
        writeln!(&mut fh, "| :-: | :-: | :-: | :-: |")?;
        for reaction in reactions {
            writeln!(&mut fh, "{}", reaction)?;
        }
        writeln!(&mut fh)?;
    }

    Ok(file)
}
