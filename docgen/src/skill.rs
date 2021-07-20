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

    fn is_skill_range(range: &reaction::CatalystRange, skill_id: usize) -> bool {
        match range {
            reaction::CatalystRange::Skill { ty, .. } => ty.0 == skill_id,
            _ => false,
        }
    }

    let reactions: Vec<_> = def
        .reaction()
        .iter()
        .filter(|reaction| {
            reaction
                .catalysts()
                .iter()
                .any(|catalyst| is_skill_range(catalyst.range(), skill_id))
        })
        .collect();

    writeln!(&mut fh, "## Required for")?;
    for reaction in reactions {
        writeln!(&mut fh, "- {}", reaction.name())?;
    }
    writeln!(&mut fh)?;

    Ok(file)
}
