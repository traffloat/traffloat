use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::KebabCase;

use super::{assets, manifest, opts};
use traffloat_vanilla::liquid;

pub fn gen_liquids(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    relativize: impl Fn(&Path) -> Result<PathBuf>,
) -> Result<Vec<manifest::Nav>> {
    let mut liquids_index = vec![manifest::Nav::Path(PathBuf::from("liquid.md"))];

    for liquid in liquid::ALL {
        let path = write_liquid(opts, assets, liquid)
            .with_context(|| format!("Writing liquid {}", liquid.name))?;
        liquids_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    {
        let mut fh = fs::File::create(opts.root_dir.join("docs/liquid.md"))
            .context("Could not create liquid.md")?;
        writeln!(&mut fh, "# List of liquid types")?;

        for liquid in liquid::ALL {
            let texture_path = opts
                .client_dir
                .join("textures")
                .join(liquid.texture.as_ref())
                .with_extension("png");
            let texture_path = texture_path
                .canonicalize()
                .with_context(|| format!("Could not canonicalize {}", texture_path.display()))?;
            writeln!(
                &mut fh,
                "- [![]({}){{ width=24 }} {}]({})",
                assets.map(&texture_path)?,
                liquid.name,
                liquid.name.to_kebab_case()
            )?;
        }
    }

    Ok(liquids_index)
}

fn write_liquid(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    liquid: &liquid::Def,
) -> Result<PathBuf> {
    let liquids_dir = opts.root_dir.join("docs/liquid");
    fs::create_dir_all(&liquids_dir).context("Could not create liquid dir")?;

    let file = liquids_dir.join(format!("{}.md", liquid.name.to_kebab_case()));
    let mut fh = fs::File::create(&file)
        .with_context(|| format!("Could not open {} for writing", file.display()))?;

    let texture_path = opts
        .client_dir
        .join("textures")
        .join(liquid.texture.as_ref())
        .with_extension("png");
    let texture_path = texture_path
        .canonicalize()
        .with_context(|| format!("Could not canonicalize {}", texture_path.display()))?;

    writeln!(&mut fh, "# {}", liquid.name)?;
    writeln!(
        &mut fh,
        "![](../{}){{ width=64 align=right }}",
        assets.map(&texture_path)?
    )?;
    writeln!(&mut fh, "> {}", liquid.summary)?;
    writeln!(&mut fh)?;
    writeln!(&mut fh, "{}", liquid.description)?;

    Ok(file)
}
