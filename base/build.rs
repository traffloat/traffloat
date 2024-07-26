//! Build protobuf files.

fn main() -> anyhow::Result<()> {
    traffloat_build_boilerplate::run_for("proto/index.proto", &["proto/"], |config| config)
}
