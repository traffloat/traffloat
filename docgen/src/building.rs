use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::KebabCase;

use super::{assets, manifest, opts};
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
            writeln!(
                &mut fh,
                "### [{}](../{}/)",
                category.title(),
                category.title().to_kebab_case()
            )?;
            writeln!(&mut fh, "{}", category.description())?;
            writeln!(&mut fh)?;
            for building in def.building() {
                if building.category().0 == category_id {
                    let texture_dir = opts
                        .client_dir
                        .join("textures")
                        .join(building.shape().texture_name());
                    let texture_dir = texture_dir.canonicalize().with_context(|| {
                        format!("Could not canonicalize {}", texture_dir.display())
                    })?;
                    writeln!(
                        &mut fh,
                        "- [![]({}) {}]({})",
                        assets.map(&texture_dir.join("xp.png"))?,
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
        assets.map(&texture_dir.join("xp.png"))?
    )?;
    writeln!(&mut fh, "> {}", building.summary())?;
    writeln!(&mut fh)?;
    writeln!(&mut fh, "{}", building.description())?;
    writeln!(&mut fh)?;

    if !building.features().is_empty() {
        writeln!(&mut fh, "## Features")?;
        for feature in building.features() {
            write_feature(&mut fh, feature)?;
        }
    }

    if !building.reactions().is_empty() {
        writeln!(&mut fh, "## Mechanisms")?;
        for (reaction_id, policy) in building.reactions() {
            let reaction = def.get_reaction(*reaction_id);
            writeln!(
                &mut fh,
                "### [{}](../../reaction/{})",
                reaction.name(),
                reaction.name().to_kebab_case()
            )?;
            writeln!(&mut fh, "{}", reaction.description())?;
            writeln!(&mut fh)?;
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
                    building::FlowPolicy::ReduceRate => "Reduce output rate",
                },
                match policy.on_overflow() {
                    building::FlowPolicy::ReduceRate => "Reduce input rate",
                },
            )?;
            writeln!(&mut fh)?;
        }
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
    }

    Ok(())
}
