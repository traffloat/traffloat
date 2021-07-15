//! Crate to generate docs.

use anyhow::{Context, Result};

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use structopt::StructOpt;

use traffloat_vanilla::buildings;

mod assets;
mod manifest;
mod opts;

fn main() -> Result<()> {
    let opts = opts::Opts::from_args();
    fs::create_dir_all(&opts.root_dir).context("Could not create --root-dir")?;
    let root_dir = opts
        .root_dir
        .canonicalize()
        .context("--root-dir could not be canonicalized")?;
    let mut assets = assets::Pool::new(root_dir.join("docs"), String::from("assets"))?;

    let mut buildings_index = Vec::new();
    let reactions_index = Vec::new();

    let relativize = |path: &Path| -> Result<PathBuf> {
        let path = path.canonicalize().context("Could not canonicalize path")?;
        let stripped = path
            .strip_prefix(&root_dir.join("docs"))
            .context("Canonicalized path is not under root_dir")?;
        Ok(stripped.to_path_buf())
    };

    for building in buildings::ALL {
        let path = write_building(&opts, &mut assets, building)
            .with_context(|| format!("Error writing building {}", building.name))?;
        buildings_index.push(manifest::Nav::Path(relativize(&path)?));
    }

    let index = vec![
        manifest::Nav::Index {
            title: String::from("Buildings"),
            items: buildings_index,
        },
        manifest::Nav::Index {
            title: String::from("Reactions"),
            items: reactions_index,
        },
    ];

    let mkdocs_yml = opts.root_dir.join("mkdocs.yml");
    let mkdocs_yml =
        fs::File::create(mkdocs_yml).context("Could not open mkdocs.yml for writing")?;
    let favicon_path = opts.client_dir.join("static/favicon.ico");
    let favicon_path = assets
        .map(&favicon_path)
        .context("Resolving favicon path")?;
    let manifest = manifest::Mkdocs {
        site_name: "Traffloat vanilla documentation",
        site_url: opts.site_url.clone().unwrap_or_else(String::new),
        use_directory_urls: opts.site_url.is_some(),
        site_author: "SOFe",
        repo_url: "https://github.com/traffloat/traffloat",
        repo_name: "traffloat/traffloat",
        copyright: "Licensed under AGPL 3.0",
        theme: manifest::Theme {
            name: "material",
            favicon: favicon_path.clone(),
            logo: favicon_path.clone(),
            features: &[],
        },
        markdown_extensions: &["attr_list"],
        nav: index,
    };
    serde_yaml::to_writer(mkdocs_yml, &manifest).context("YAML formatting error")?;

    Ok(())
}

fn write_building(
    opts: &opts::Opts,
    assets: &mut assets::Pool,
    building: &buildings::Def,
) -> Result<PathBuf> {
    use heck::KebabCase;

    let buildings_dir = opts.root_dir.join("docs/buildings");
    fs::create_dir_all(&buildings_dir).context("Could not create buildings dir")?;

    let file = buildings_dir.join(format!("{}.md", building.name.to_kebab_case()));
    let mut fh = fs::File::create(&file)
        .with_context(|| format!("Could not open {} for writing", file.display()))?;

    let texture_dir = opts
        .client_dir
        .join("textures")
        .join(building.shape.texture);
    let texture_dir = texture_dir
        .canonicalize()
        .with_context(|| format!("Could not canonicalize {}", texture_dir.display()))?;

    writeln!(&mut fh, "# {}", building.name)?;
    writeln!(
        &mut fh,
        "![](../{}){{ width=64 align=right }}",
        assets.map(&texture_dir.join("xp.png"))?
    )?;
    writeln!(&mut fh, "> {}", building.summary)?;
    writeln!(&mut fh)?;
    writeln!(&mut fh, "{}", building.description)?;

    Ok(file)
}
