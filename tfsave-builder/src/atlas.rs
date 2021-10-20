use std::convert::TryInto;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::sync::atomic;
use std::sync::atomic::AtomicU32;

use anyhow::{Context, Result};
use arcstr::ArcStr;
use safety::Safety;
use serde::Deserialize;
use tiny_skia::{Pixmap, PixmapPaint};
use traffloat_def::atlas;
use traffloat_types::geometry;

use crate::{Timer, TimerStart};

pub(crate) fn generate(
    input: &Path,
    output: &Path,
    render_timer: &Timer,
    downscale_timer: &Timer,
    save_timer: &Timer,
    def: &atlas::Def,
    next_texture_id: &AtomicU32,
    skip_svg: bool,
    mut register_icon_texture_id: impl FnMut(&ArcStr, u32),
    mut register_model_texture_id: impl FnMut(&ArcStr, u32, geometry::Unit),
) -> Result<()> {
    let input = input.canonicalize().context("Cannot canonicalize input file")?;
    let dir = input.parent().context("Regular file has no parent")?;
    let mut dir = dir.join(def.dir());
    dir = dir
        .canonicalize()
        .with_context(|| format!("Cannont canonicalize atlas path {}", dir.display()))?;

    for variant in def.variants() {
        let variant_dir = output.join(variant.name().as_str());
        if !variant_dir.exists() {
            fs::create_dir(&variant_dir).with_context(|| {
                format!("Cannot create variant directory {}", variant_dir.display())
            })?;
        }
    }

    log::info!("Generating atlas for {}", dir.display());

    let options = usvg::Options::default();
    let options = options.to_ref();

    let max_dim = 1024;

    for file in fs::read_dir(&dir).context("Scanning directory list")? {
        let file = file.context("Scanning directory list")?;
        let path = file.path();
        let texture_id = next_texture_id.fetch_add(1, atomic::Ordering::SeqCst);

        if path.extension() == Some(OsStr::new("svg")) {
            log::debug!("Rendering SVG file {}", path.display());

            let pixmap = render_svg(&path, max_dim, &options, skip_svg, render_timer.start())
                .with_context(|| format!("Rendering SVG file {}", path.display()))?;
            if !skip_svg {
                write_variants(
                    def.variants(),
                    pixmap,
                    1, // singleton spritesheet
                    max_dim,
                    output,
                    downscale_timer,
                    save_timer,
                    texture_id,
                )
                .context("Emitting PNG output")?;
            }

            let name = path.file_name().expect("Path has filename");
            let name = name.to_str().context("Texture name must be ASCII only")?;
            let name = name.strip_suffix(".svg").expect("Path has svg file extension");
            register_icon_texture_id(&ArcStr::from(name), texture_id);
        } else if path.join("model.toml").exists() {
            log::debug!("Rendering SVG model {}", path.display());

            let model = fs::read_to_string(path.join("model.toml"))
                .with_context(|| format!("Reading {}/model.toml", path.display()))?;
            let model: ModelToml = toml::from_str(&model)
                .with_context(|| format!("Parsing {}/model.toml", path.display()))?;

            let dimension = model.unit.spritesheet_side() * max_dim;
            let mut pixmap = Pixmap::new(dimension, dimension).context("Creating pixmap buffer")?;
            for (sprite_id, file) in model.unit.sprite_names().iter().enumerate() {
                let svg_path = path.join(file).with_extension("svg");
                let sprite_pixmap =
                    render_svg(&svg_path, max_dim, &options, skip_svg, render_timer.start())
                        .with_context(|| format!("Rendering SVG file {}", svg_path.display()))?;

                let (x, y) = model
                    .unit
                    .sprite_coords(sprite_id.try_into().expect("Sprite count is a small number"));
                pixmap
                    .draw_pixmap(
                        (x * max_dim).homosign(),
                        (y * max_dim).homosign(),
                        sprite_pixmap.as_ref(),
                        &PixmapPaint::default(),
                        tiny_skia::Transform::identity(),
                        None,
                    )
                    .context("Copying pixmap buffer")?;
            }

            if !skip_svg {
                write_variants(
                    def.variants(),
                    pixmap,
                    model.unit.spritesheet_side(),
                    dimension,
                    output,
                    downscale_timer,
                    save_timer,
                    texture_id,
                )
                .context("Emitting PNG output")?;
            }

            let name = path.file_name().expect("Path has filename");
            let name = name.to_str().context("Texture name must be ASCII only")?;
            register_model_texture_id(&ArcStr::from(name), texture_id, model.unit);
        }
    }

    Ok(())
}

fn write_variants(
    variants: &[atlas::Variant],
    pixmap: Pixmap,
    side_sprite_count: u32,
    pixmap_size: u32,
    output: &Path,
    downscale_timer: &Timer,
    save_timer: &Timer,
    texture_id: u32,
) -> Result<()> {
    for variant in variants {
        let out_png = output
            .join(variant.name().as_str())
            .join(format!("{:08x}", texture_id))
            .with_extension("png");
        log::debug!("Saving downscaled variant {}", out_png.display());

        let downscaled_map = {
            let _timer = downscale_timer.start();
            downscale(&pixmap, pixmap_size, variant.dimension() * side_sprite_count)
                .context("Downscaling sprite")?
        };
        {
            let _timer = save_timer.start();
            downscaled_map
                .save_png(&out_png)
                .with_context(|| format!("Saving atlas as PNG at {}", out_png.display()))?;
        }
    }

    Ok(())
}

fn downscale(map: &Pixmap, old_size: u32, new_size: u32) -> Result<Pixmap> {
    debug_assert!(new_size <= old_size, "new_size ({}) > old_size ({})", new_size, old_size);
    let ratio = new_size.small_float() / old_size.small_float();
    let side = (map.width().small_float() * ratio).trunc_int();
    let mut submap = Pixmap::new(side, side).context("Creating downscaled pixmap buffer")?;
    submap
        .draw_pixmap(
            0,
            0,
            map.as_ref(),
            &PixmapPaint::default(),
            tiny_skia::Transform::from_scale(ratio, ratio),
            None,
        )
        .context("Downscaling pixmap buffer")?;
    Ok(submap)
}

fn render_svg(
    path: &Path,
    dimension: u32,
    options: &usvg::OptionsRef,
    skip_svg: bool,
    _timer: TimerStart<'_>,
) -> Result<Pixmap> {
    let data = fs::read(path).context("Reading SVG")?;

    let mut pixmap = Pixmap::new(dimension, dimension).context("Creating pixmap buffer")?;

    if !skip_svg {
        let tree = usvg::Tree::from_data(&data, options).context("Parsing SVG")?;
        resvg::render(&tree, usvg::FitTo::Size(dimension, dimension), pixmap.as_mut())
            .context("Rendering SVG")?;
    }

    Ok(pixmap)
}

#[derive(Deserialize)]
struct ModelToml {
    unit: geometry::Unit,
}
