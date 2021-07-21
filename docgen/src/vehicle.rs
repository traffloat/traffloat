use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::KebabCase;

use super::{assets, manifest, opts};
use traffloat_types::def::{vehicle, GameDefinition};

pub fn gen_vehicles(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    relativize: impl Fn(&Path) -> Result<PathBuf>,
    def: &GameDefinition,
) -> Result<Vec<manifest::Nav>> {
    let mut vehicles_index = vec![manifest::Nav::Path(PathBuf::from("vehicle.md"))];

    for (vehicle_id, vehicle) in def.vehicle().iter().enumerate() {
        let path = write_vehicle(opts, assets, vehicle_id, vehicle, def)
            .with_context(|| format!("Writing vehicle {}", vehicle.name()))?;
        vehicles_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    {
        let mut fh = fs::File::create(opts.root_dir.join("docs/vehicle.md"))
            .context("Could not create vehicle.md")?;
        writeln!(&mut fh, "{}", include_str!("vehicle.md"))?;
        writeln!(&mut fh, "## List of vehicle types")?;

        for vehicle in def.vehicle() {
            let texture_path = opts
                .client_dir
                .join("textures")
                .join(vehicle.texture())
                .with_extension("png");
            let texture_path = texture_path
                .canonicalize()
                .with_context(|| format!("Could not canonicalize {}", texture_path.display()))?;
            writeln!(
                &mut fh,
                "- [{}]({})",
                vehicle.name(),
                vehicle.name().to_kebab_case()
            )?;
        }
    }

    Ok(vehicles_index)
}

fn write_vehicle(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    vehicle_id: usize,
    vehicle: &vehicle::Type,
    def: &GameDefinition,
) -> Result<PathBuf> {
    let vehicles_dir = opts.root_dir.join("docs/vehicle");
    fs::create_dir_all(&vehicles_dir).context("Could not create vehicle dir")?;

    let file = vehicles_dir.join(format!("{}.md", vehicle.name().to_kebab_case()));
    let mut fh = fs::File::create(&file)
        .with_context(|| format!("Could not open {} for writing", file.display()))?;

    writeln!(&mut fh, "# {}", vehicle.name())?;
    writeln!(&mut fh, "{}", vehicle.description())?;
    writeln!(&mut fh)?;

    let texture_path = opts
        .client_dir
        .join("textures")
        .join(vehicle.texture())
        .with_extension("png");
    let texture_path = texture_path
        .canonicalize()
        .with_context(|| format!("Could not canonicalize {}", texture_path.display()))?;
    writeln!(
        &mut fh,
        "![](../{}){{ width=64 align=right }}",
        assets.map(&texture_path)?
    )?;
    writeln!(&mut fh)?;

    writeln!(&mut fh, "| Property | Value |")?;
    writeln!(&mut fh, "| :-: | :-: |")?;
    writeln!(&mut fh, "| Speed | {} |", vehicle.speed())?;
    writeln!(&mut fh, "| Cargo capacity | {} |", vehicle.capacity())?;
    writeln!(&mut fh, "| Passenger capacity | {} |", vehicle.passengers())?;

    let skill = def.get_skill(vehicle.skill().skill());
    writeln!(
        &mut fh,
        "| When driver [{}](../../skill/{}) is below {} | {}x |",
        skill.name(),
        skill.name().to_kebab_case(),
        vehicle.skill().levels().start,
        vehicle.skill().multipliers().underflow(),
    )?;
    writeln!(
        &mut fh,
        "| When driver [{}](../../skill/{}) is between {} and {} | {}x to {}x speed |",
        skill.name(),
        skill.name().to_kebab_case(),
        vehicle.skill().levels().start,
        vehicle.skill().levels().end,
        vehicle.skill().multipliers().min(),
        vehicle.skill().multipliers().max(),
    )?;
    writeln!(
        &mut fh,
        "| When driver [{}](../../skill/{}) is above {} | {}x speed |",
        skill.name(),
        skill.name().to_kebab_case(),
        vehicle.skill().levels().end,
        vehicle.skill().multipliers().overflow(),
    )?;

    Ok(file)
}
