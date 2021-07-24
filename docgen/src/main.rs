//! Crate to generate docs.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use structopt::StructOpt;

mod assets;
mod building;
mod cargo;
mod electricity;
mod gas;
mod liquid;
mod manifest;
mod opts;
mod reaction;
mod skill;
mod vehicle;

fn main() -> Result<()> {
    let opts = opts::Opts::from_args();
    fs::create_dir_all(&opts.root_dir).context("Could not create --root-dir")?;
    let root_dir = opts
        .root_dir
        .canonicalize()
        .context("--root-dir could not be canonicalized")?;
    let mut assets = assets::Pool::new(root_dir.join("docs"), String::from("assets"))?;

    let relativize = |path: &Path| {
        let path = path.canonicalize().context("Could not canonicalize path")?;
        let stripped = path
            .strip_prefix(&root_dir.join("docs"))
            .context("Canonicalized path is not under root_dir")?;
        Ok(stripped.to_path_buf())
    };

    let (def, _, _) = traffloat_vanilla::get();

    let buildings_index = building::gen_buildings(&opts, &mut assets, relativize, &def)
        .context("Generating building guide")?;
    let cargos_index = cargo::gen_cargos(&opts, &mut assets, relativize, &def)
        .context("Generating cargo guide")?;
    let gases_index =
        gas::gen_gases(&opts, &mut assets, relativize, &def).context("Generating gases guide")?;
    let liquids_index = liquid::gen_liquids(&opts, &mut assets, relativize, &def)
        .context("Generating liquid guide")?;
    let skills_index = skill::gen_skills(&opts, &mut assets, relativize, &def)
        .context("Generating skill guide")?;
    let vehicles_index = vehicle::gen_vehicles(&opts, &mut assets, relativize, &def)
        .context("Generating vehicle guide")?;
    electricity::gen_electricity(&opts, &mut assets, relativize, &def)
        .context("Generating electricity guide")?;

    {
        let docs_dir = opts.root_dir.join("docs");
        fs::write(docs_dir.join("controls.md"), include_str!("controls.md"))
            .context("Copying file")?;
        fs::write(docs_dir.join("corridor.md"), include_str!("corridor.md"))
            .context("Copying file")?;
        fs::write(docs_dir.join("index.md"), include_str!("index.md")).context("Copying file")?;
        fs::write(
            docs_dir.join("population.md"),
            include_str!("population.md"),
        )
        .context("Copying file")?;
        fs::write(docs_dir.join("housing.md"), include_str!("housing.md"))
            .context("Copying file")?;
        fs::write(docs_dir.join("happiness.md"), include_str!("happiness.md"))
            .context("Copying file")?;
    }

    let index = vec![
        manifest::Nav::Path(PathBuf::from("index.md")),
        manifest::Nav::Path(PathBuf::from("controls.md")),
        manifest::Nav::Index {
            title: String::from("Buildings"),
            items: buildings_index,
        },
        manifest::Nav::Path(PathBuf::from("corridor.md")),
        manifest::Nav::Index {
            title: String::from("Population"),
            items: vec![
                manifest::Nav::Path(PathBuf::from("population.md")),
                manifest::Nav::Path(PathBuf::from("housing.md")),
                manifest::Nav::Path(PathBuf::from("happiness.md")),
                manifest::Nav::Index {
                    title: String::from("Skill"),
                    items: skills_index,
                },
            ],
        },
        manifest::Nav::Index {
            title: String::from("Cargo"),
            items: cargos_index,
        },
        manifest::Nav::Index {
            title: String::from("Gases"),
            items: gases_index,
        },
        manifest::Nav::Index {
            title: String::from("Liquids"),
            items: liquids_index,
        },
        manifest::Nav::Path(PathBuf::from("electricity.md")),
        manifest::Nav::Index {
            title: String::from("Vehicle"),
            items: vehicles_index,
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
        site_name: "Traffloat user manual",
        site_url: opts.site_url.clone().unwrap_or_else(String::new),
        use_directory_urls: opts.site_url.is_some(),
        site_author: "SOFe",
        repo_url: "https://github.com/traffloat/traffloat",
        repo_name: "traffloat/traffloat",
        edit_uri: "https://github.com/traffloat/traffloat/edit/master/docgen/src/",
        copyright: "Licensed under AGPL 3.0",
        theme: manifest::Theme {
            name: "material",
            favicon: favicon_path.clone(),
            logo: favicon_path,
            features: &[],
            palette: serde_json::json! {
                [
                    {
                        "scheme": "default",
                        "toggle": {
                            "icon": "material/toggle-switch-off-outline",
                            "name": "Switch to dark mode",
                        }
                    },
                    {
                        "scheme": "slate",
                        "toggle": {
                            "icon": "material/toggle-switch",
                            "name": "Switch to light mode",
                        }
                    }
                ]
            },
        },
        markdown_extensions: &["attr_list"],
        nav: index,
    };
    serde_yaml::to_writer(mkdocs_yml, &manifest).context("YAML formatting error")?;

    Ok(())
}

fn building_texture(dir: &Path) -> PathBuf {
    let mut file = dir.join("xp.svg");
    if file.is_file() {
        return file;
    }

    file.set_file_name("curved.svg");
    if file.is_file() {
        return file;
    }

    panic!("Building texture at {} is unsupported", dir.display())
}
