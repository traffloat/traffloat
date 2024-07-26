use std::env;

use anyhow::Context;

/// Build protobuf files for the standard structure.
pub fn run() -> anyhow::Result<()> {
    run_for("src/saves.proto", &["src", "../base/proto"], |config| {
        config.extern_path(".traffloat.base", "::traffloat_base::proto")
    })
}

/// Build protobuf files for specific paths.
pub fn run_for(
    entrypoint: &str,
    includes: &[&str],
    patch_config: impl FnOnce(&mut prost_build::Config) -> &mut prost_build::Config,
) -> anyhow::Result<()> {
    let base_crate_name = if env::var("CARGO_PKG_NAME").is_ok_and(|value| value == "traffloat-base") {
        "crate"
    } else {
        "::traffloat_base"
    };

    let mut config = prost_build::Config::new();
    config.type_attribute(".", format!(r##"
        #[derive({base_crate_name}::serde::Serialize, {base_crate_name}::serde::Deserialize)]
        #[allow(missing_docs)]
        #[serde(crate = "{base_crate_name}::serde", rename_all = "camelCase")]
        "##));
    patch_config(&mut config);
    config.compile_protos(&[entrypoint], includes).context("compile protobuf")
}
