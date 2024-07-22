//! Build protobuf files.

fn main() -> anyhow::Result<()> {
    prost_build::compile_protos(&["src/saves.proto"], &["../base/proto/", "src/"])?;
    Ok(())
}
