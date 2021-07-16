use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::KebabCase;

use super::{assets, manifest, opts};
use traffloat_vanilla::gas;

pub fn gen_gases(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    relativize: impl Fn(&Path) -> Result<PathBuf>,
) -> Result<Vec<manifest::Nav>> {
    let mut gases_index = vec![manifest::Nav::Path(PathBuf::from("gas.md"))];

    for gas in gas::ALL {
        let path =
            write_gas(opts, assets, gas).with_context(|| format!("Writing gas {}", gas.name))?;
        gases_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    {
        let mut fh = fs::File::create(opts.root_dir.join("docs/gas.md"))
            .context("Could not create gas.md")?;
        writeln!(&mut fh, "# List of gas types")?;

        for gas in gas::ALL {
            let texture_path = opts
                .client_dir
                .join("textures")
                .join(gas.texture.as_ref())
                .with_extension("png");
            let texture_path = texture_path
                .canonicalize()
                .with_context(|| format!("Could not canonicalize {}", texture_path.display()))?;
            writeln!(
                &mut fh,
                "- [![]({}){{ width=24 }} {}]({})",
                assets.map(&texture_path)?,
                gas.name,
                gas.name.to_kebab_case()
            )?;
        }
    }

    Ok(gases_index)
}

fn write_gas(opts: &opts::Opts, assets: &mut assets::Pool, gas: &gas::Def) -> Result<PathBuf> {
    let gases_dir = opts.root_dir.join("docs/gas");
    fs::create_dir_all(&gases_dir).context("Could not create gas dir")?;

    let file = gases_dir.join(format!("{}.md", gas.name.to_kebab_case()));
    let mut fh = fs::File::create(&file)
        .with_context(|| format!("Could not open {} for writing", file.display()))?;

    let texture_path = opts
        .client_dir
        .join("textures")
        .join(gas.texture.as_ref())
        .with_extension("png");
    let texture_path = texture_path
        .canonicalize()
        .with_context(|| format!("Could not canonicalize {}", texture_path.display()))?;

    writeln!(&mut fh, "# {}", gas.name)?;
    writeln!(
        &mut fh,
        "![](../{}){{ width=64 align=right }}",
        assets.map(&texture_path)?
    )?;
    writeln!(&mut fh, "> {}", gas.summary)?;
    writeln!(&mut fh)?;
    writeln!(&mut fh, "{}", gas.description)?;

    Ok(file)
}
