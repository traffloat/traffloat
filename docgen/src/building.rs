use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::KebabCase;

use super::{assets, manifest, opts};
use crate::reaction;
use traffloat_types::def::{building, GameDefinition};

pub fn gen_buildings(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    relativize: impl Fn(&Path) -> Result<PathBuf>,
    def: &GameDefinition,
) -> Result<Vec<manifest::Nav>> {
    let mut buildings_index = vec![manifest::Nav::Path(PathBuf::from("building.md"))];

    for building in def.building() {
        let path = write_building(opts, assets, building, def)
            .with_context(|| format!("Writing building {}", building.name()))?;
        buildings_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    {
        let mut fh = fs::File::create(opts.root_dir.join("docs/building.md"))
            .context("Could not create building.md")?;
        writeln!(&mut fh, "{}", include_str!("building.md"))?;
        writeln!(&mut fh, "## List of buildings")?;

        for (category_id, category) in def.building_cats().iter().enumerate() {
            writeln!(&mut fh, "### {}", category.title())?;
            writeln!(&mut fh, "{}", category.description())?;
            writeln!(&mut fh)?;
            for building in def.building() {
                if building.category().0 == category_id {
                    let texture_dir = opts
                        .client_dir
                        .join("textures")
                        .join(building.shape().texture_name().as_str());
                    let texture_dir = texture_dir.canonicalize().with_context(|| {
                        format!("Could not canonicalize {}", texture_dir.display())
                    })?;
                    writeln!(
                        &mut fh,
                        "- [![]({}){{ width=20 }} {}]({})",
                        assets.map(&super::building_texture(&texture_dir))?,
                        building.name(),
                        building.name().to_kebab_case()
                    )?;
                }
            }
        }
    }

    Ok(buildings_index)
}

fn write_building(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    building: &building::Type,
    def: &GameDefinition,
) -> Result<PathBuf> {
    let buildings_dir = opts.root_dir.join("docs/building");
    fs::create_dir_all(&buildings_dir).context("Could not create building dir")?;

    let file = buildings_dir.join(format!("{}.md", building.name().to_kebab_case()));
    let mut fh = fs::File::create(&file)
        .with_context(|| format!("Could not open {} for writing", file.display()))?;

    let texture_dir = opts
        .client_dir
        .join("textures")
        .join(building.shape().texture_name().as_str());
    let texture_dir = texture_dir
        .canonicalize()
        .with_context(|| format!("Could not canonicalize {}", texture_dir.display()))?;

    writeln!(&mut fh, "# {}", building.name())?;
    writeln!(
        &mut fh,
        "![](../{}){{ width=64 align=right }}",
        assets.map(&super::building_texture(&texture_dir))?
    )?;
    writeln!(&mut fh, "> {}", building.summary())?;
    writeln!(&mut fh)?;
    writeln!(&mut fh, "{}", building.description())?;
    writeln!(&mut fh)?;

    writeln!(&mut fh, "| Property | Value |")?;
    writeln!(&mut fh, "| :-: | :-: |")?;
    writeln!(&mut fh, "| Hitpoints | {} |", building.hitpoint())?;
    writeln!(
        &mut fh,
        "| Cargo storage | {} |",
        building.storage().cargo()
    )?;
    writeln!(
        &mut fh,
        "| Liquid storage | {} |",
        building.storage().liquid()
    )?;
    writeln!(&mut fh, "| Gas storage | {} |", building.storage().gas())?;
    writeln!(&mut fh)?;

    if !building.features().is_empty() || building.reactions().is_empty() {
        writeln!(&mut fh, "## Mechanisms")?;
    }

    for feature in building.features() {
        write_feature(&mut fh, feature)?;
    }

    for (reaction_id, policy) in building.reactions() {
        let reaction = def.get_reaction(*reaction_id);
        reaction::write_reaction(opts, assets, reaction, def, &mut fh, "###")
            .context("Writing reaction information")?;
        writeln!(&mut fh)?;
        writeln!(&mut fh, "#### Rate control")?;
        writeln!(
            &mut fh,
            "| Player can manually restrict rate | When inputs underflow | When outputs overflow |"
        )?;
        writeln!(&mut fh, "| :-: | :-: | :-: |")?;
        writeln!(
            &mut fh,
            "| {} | {} | {} |",
            if policy.configurable() { "Yes" } else { "No" },
            match policy.on_underflow() {
                building::FlowPolicy::ReduceRate => "Reduce rate",
            },
            match policy.on_overflow() {
                building::FlowPolicy::ReduceRate => "Reduce rate",
            },
        )?;
        writeln!(&mut fh)?;
    }

    Ok(file)
}

fn write_feature(mut fh: impl Write, feature: &building::ExtraFeature) -> Result<()> {
    match feature {
        building::ExtraFeature::Core => {
            writeln!(&mut fh, "### Core")?;
            writeln!(
                &mut fh,
                "This is a core building. Destruction of this building will end the game."
            )?;
            writeln!(&mut fh)?;
        }
        building::ExtraFeature::ProvidesHousing(capacity) => {
            writeln!(&mut fh, "### Housing ({} inhabitants)", capacity)?;
            writeln!(
                &mut fh,
                "This building provides {} [housing capacity](../../happiness).",
                capacity
            )?;
            writeln!(
                &mut fh,
                "Inhabitants assigned to this building will be affected by"
            )?;
            writeln!(
                &mut fh,
                "the happiness-related mechanisms of this building, such as food."
            )?;
            writeln!(&mut fh)?;
        }
        building::ExtraFeature::RailTerminal(force) => {
            writeln!(&mut fh, "### Rail terminal")?;
            writeln!(&mut fh, "Vehicles in adjacent [corridors](../../corridor#vehicles) are powered by an extra {}.", force)?;
            writeln!(&mut fh)?;
        }
        building::ExtraFeature::LiquidPump(force) => {
            writeln!(&mut fh, "### Liquid pump")?;
            writeln!(
                &mut fh,
                "Pipes in adjacent [corridors](../../corridor#liquids) are powered by an extra {}.",
                force
            )?;
            writeln!(&mut fh)?;
        }
        building::ExtraFeature::GasPump(force) => {
            writeln!(&mut fh, "### Gas fan")?;
            writeln!(&mut fh, "Fans can be installed on adjacent [corridors](../../corridor#gase) to speed up gas diffusion.")?;
            writeln!(
                &mut fh,
                "Each fan provides up to {} of pumping force.",
                force
            )?;
            writeln!(&mut fh)?;
        }
        building::ExtraFeature::SecureEntry {
            min_happiness,
            breach_probability,
        } => {
            writeln!(&mut fh, "### Entry security")?;
            writeln!(
                &mut fh,
                "Inhabitants entering the building must have at least {} happiness.",
                min_happiness
            )?;
            if *breach_probability > 0. {
                writeln!(
                    &mut fh,
                    "However, inhabitants with low happiness may manage to sneak into the building with {} probability.",
                    breach_probability,
                )?;
            }
            writeln!(&mut fh)?;
        }
        building::ExtraFeature::SecureExit {
            min_happiness,
            breach_probability,
        } => {
            writeln!(&mut fh, "### Exit security")?;
            writeln!(
                &mut fh,
                "Inhabitants exiting the building must have at least {} happiness.",
                min_happiness
            )?;
            if *breach_probability > 0. {
                writeln!(
                    &mut fh,
                    "However, inhabitants with low happiness may manage to sneak out of the building with {} probability.",
                    breach_probability,
                )?;
            }
            writeln!(&mut fh)?;
        }
    }

    Ok(())
}
