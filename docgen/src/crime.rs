use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::KebabCase;

use super::{assets, manifest, opts};
use traffloat_types::def::{
    crime::{self, InhabitantCriterion},
    GameDefinition,
};

pub fn gen_crimes(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    relativize: impl Fn(&Path) -> Result<PathBuf>,
    def: &GameDefinition,
) -> Result<Vec<manifest::Nav>> {
    let mut crimes_index = vec![manifest::Nav::Path(PathBuf::from("crime.md"))];

    for crime in def.crime().iter() {
        let path = write_crime(opts, assets, crime, def)
            .with_context(|| format!("Writing crime {}", crime.name()))?;
        crimes_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    {
        let mut fh = fs::File::create(opts.root_dir.join("docs/crime.md"))
            .context("Could not create crime.md")?;
        writeln!(&mut fh, "{}", include_str!("crime.md"))?;
        writeln!(&mut fh, "## List of crimes")?;

        for crime in def.crime() {
            writeln!(
                &mut fh,
                "- [{}]({})",
                crime.name(),
                crime.name().to_kebab_case()
            )?;
        }
    }

    Ok(crimes_index)
}

fn write_crime(
    opts: &opts::Opts,
    _assets: &mut assets::Pool,
    crime: &crime::Type,
    def: &GameDefinition,
) -> Result<PathBuf> {
    let crimes_dir = opts.root_dir.join("docs/crime");
    fs::create_dir_all(&crimes_dir).context("Could not create crime dir")?;

    let file = crimes_dir.join(format!("{}.md", crime.name().to_kebab_case()));
    let mut fh = fs::File::create(&file)
        .with_context(|| format!("Could not open {} for writing", file.display()))?;

    writeln!(&mut fh, "# {}", crime.name())?;
    writeln!(&mut fh, "{}", crime.description())?;
    writeln!(&mut fh)?;

    let trigger_skill = def.get_skill(crime.trigger_skill());

    writeln!(&mut fh, "| Property | Value |")?;
    writeln!(&mut fh, "| :-: | :-: |")?;
    writeln!(
        &mut fh,
        "| Trigger {} level | {} to {} |",
        trigger_skill.name(),
        crime.trigger_skill_range().start,
        crime.trigger_skill_range().end,
    )?;
    writeln!(
        &mut fh,
        "| Average commit frequency per in trigger range | {} seconds |",
        1. / crime.probability()
    )?;
    for &(ty, delta) in crime.skill_change() {
        let skill = def.get_skill(ty);
        writeln!(
            &mut fh,
            "| {} change after committing crime | {:+} |",
            skill.name(),
            delta
        )?;
    }
    writeln!(&mut fh)?;

    writeln!(&mut fh, "## What happens?")?;
    match crime.action() {
        crime::Action::InhabitantTheft(size) => {
            writeln!(
                &mut fh,
                "When an outlaw decides to commit this crime, \
                they first select another random inhabitant in the same building or vehicle. \
                If they are alone, they continue what they have been doing \
                until they come across with one. \
                Then they remove up to {} cargo carried by the selected inhabitant, \
                then continue with their original task. \
                The removed cargo is permanently lost.",
                size
            )?;
        }
        crime::Action::VehicleTheft(size) => {
            writeln!(
                &mut fh,
                "When an outlaw decides to commit this crime, \
                they continue what they have been doing \
                until they are in the same building as a vehicle that carries cargo. \
                Then they remove up to {} cargo carried by the vehicle \
                onto themselves, then continue with their original task. \
                The removed cargo is permanently lost.",
                size
            )?;
        }
        crime::Action::NodeTheft(size) => {
            writeln!(
                &mut fh,
                "When an outlaw decides to commit this crime, \
                they choose a building with cargo randomly.
                Then they start moving to the building using any usual commute. \
                Upon arrival at the building, \
                they remove up to {} cargo stored in the building. \
                Then they choose another accessible building in the colony \
                and start moving to that building using any commute. \
                The removed cargo is permanently lost.",
                size
            )?;
        }
        crime::Action::Antagonize(crit, skill, delta) => {
            writeln!(
                &mut fh,
                "When an outlaw decides to commit this crime, \
                they select {}. \
                Then they start chasing the selected inhabitant. \
                Upon entering the same building or vehicle, \
                the selected inhabitant reduces {} by {}. \
                Both inhabitants continue with their original task afterwards.",
                InhabitantCriterionFormat(def, *crit),
                def.get_skill(*skill).name(),
                delta
            )?;
        }
        crime::Action::Arson => {
            writeln!(
                &mut fh,
                "When an outlaw decides to commit this crime, \
                they search for a building storing flammable items \
                and carry as much as they can. \
                Then they select any random building to set fire. \
                They start moving to the building using any usual commute. \
                Upon arrival at the building,
                they remove the flammable items they carry \
                and start burning effect on the building."
            )?;
        }
    }

    Ok(file)
}

struct InhabitantCriterionFormat<'t>(&'t GameDefinition, InhabitantCriterion);

impl<'t> fmt::Display for InhabitantCriterionFormat<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.1 {
            InhabitantCriterion::HighestSkill(ty) => write!(
                f,
                "the inhabitant with the highest {}",
                self.0.get_skill(ty).name()
            ),
        }
    }
}
