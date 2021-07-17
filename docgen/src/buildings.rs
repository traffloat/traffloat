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
    let mut buildings_index = vec![manifest::Nav::Path(PathBuf::from("buildings.md"))];

    for building in def.building() {
        let path = write_building(opts, assets, building)
            .with_context(|| format!("Writing building {}", building.name()))?;
        buildings_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    {
        let mut fh = fs::File::create(opts.root_dir.join("docs/buildings.md"))
            .context("Could not create buildings.md")?;
        writeln!(&mut fh, "{}", include_str!("buildings.md"))?;
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
                    let texture_dir = opts.client_dir.join("textures").join(building.texture());
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
) -> Result<PathBuf> {
    let buildings_dir = opts.root_dir.join("docs/buildings");
    fs::create_dir_all(&buildings_dir).context("Could not create buildings dir")?;

    let file = buildings_dir.join(format!("{}.md", building.name().to_kebab_case()));
    let mut fh = fs::File::create(&file)
        .with_context(|| format!("Could not open {} for writing", file.display()))?;

    let texture_dir = opts
        .client_dir
        .join("textures")
        .join(building.texture().as_str());
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

    Ok(file)
}
