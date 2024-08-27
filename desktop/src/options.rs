use std::path::PathBuf;

use bevy::ecs::system::Resource;

#[derive(clap::Parser, Resource, Default)]
#[command(name = "traffloat", version = traffloat_version::VERSION, about)]
pub struct Options {
    pub save_file: Option<PathBuf>,
    #[clap(long, default_value = "assets/")]
    pub asset_dir: PathBuf,
}

impl Options {
    #[cfg(target_family = "wasm")]
    pub fn parse_by_platform() -> Result<Self, String> { Ok(Self::default()) }

    #[cfg(not(target_family = "wasm"))]
    pub fn parse_by_platform() -> Result<Self, String> {
        use std::fs;

        let mut options = <Self as clap::Parser>::parse();
        let asset_dir = match fs::canonicalize(&options.asset_dir) {
            Ok(asset_dir) => asset_dir,
            Err(err) => {
                return Err(format!(
                    "Asset directory {} is not canonicalizable: {err}",
                    options.asset_dir.display()
                ))
            }
        };
        options.asset_dir = asset_dir;
        Ok(options)
    }
}
