use std::convert::TryInto;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::sync::atomic;
use std::sync::atomic::AtomicU32;

use anyhow::{Context, Result};
use arcstr::ArcStr;
use serde::Deserialize;
use tiny_skia::{Pixmap, PixmapPaint};
use traffloat_def::atlas::{self, SpritesheetId};
use traffloat_types::geometry;
use typed_builder::TypedBuilder;
use xias::Xias;
use xylem::DefaultContext;

use crate::{Timer, TimerStart};

#[derive(TypedBuilder)]
pub(crate) struct GenerateArgs<'t, RegisterIcon, RegisterModel>
where
    RegisterIcon: FnMut(&mut DefaultContext, &ArcStr, SpritesheetId),
    RegisterModel: FnMut(&mut DefaultContext, &ArcStr, SpritesheetId, geometry::Unit),
{
    input:                     &'t Path,
    output:                    &'t Path,
    render_timer:              &'t Timer,
    downscale_timer:           &'t Timer,
    save_timer:                &'t Timer,
    atlas:                     &'t atlas::Def,
    next_texture_id:           &'t AtomicU32,
    skip_svg:                  bool,
    register_icon_texture_id:  RegisterIcon,
    register_model_texture_id: RegisterModel,
}

pub(crate) fn generate<RegisterIcon, RegisterModel>(
    context: &mut DefaultContext,
    mut args: GenerateArgs<'_, RegisterIcon, RegisterModel>,
) -> Result<()>
where
    RegisterIcon: FnMut(&mut DefaultContext, &ArcStr, SpritesheetId),
    RegisterModel: FnMut(&mut DefaultContext, &ArcStr, SpritesheetId, geometry::Unit),
{
    let input = args.input.canonicalize().context("Cannot canonicalize input file")?;
    let dir = input.parent().context("Regular file has no parent")?;
    let mut dir = dir.join(args.atlas.dir());
    dir = dir
        .canonicalize()
        .with_context(|| format!("Cannont canonicalize atlas path {}", dir.display()))?;

    log::info!("Generating atlas for {}", dir.display());

    let options = usvg::Options::default();
    let options = options.to_ref();

    let max_dim = 1024;

    for file in fs::read_dir(&dir).context("Scanning directory list")? {
        let file = file.context("Scanning directory list")?;
        let path = file.path();
        let texture_id = args.next_texture_id.fetch_add(1, atomic::Ordering::SeqCst);
        let texture_id = SpritesheetId::new(texture_id);

        if path.extension() == Some(OsStr::new("svg")) {
            log::debug!("Rendering SVG file {}", path.display());

            let pixmap =
                render_svg(&path, max_dim, &options, args.skip_svg, args.render_timer.start())
                    .with_context(|| format!("Rendering SVG file {}", path.display()))?;
            if !args.skip_svg {
                let args = WriteVariantsArgs::builder()
                    .variants(args.atlas.variants())
                    .pixmap(pixmap)
                    .side_sprite_count(1) // singleton spritesheet
                    .pixmap_size(max_dim)
                    .output(args.output)
                    .downscale_timer(args.downscale_timer)
                    .save_timer(args.save_timer)
                    .texture_id(texture_id)
                    .build();
                write_variants(args).context("Emitting PNG output")?;
            }

            let name = path.file_name().expect("Path has filename");
            let name = name.to_str().context("Texture name must be ASCII only")?;
            let name = name.strip_suffix(".svg").expect("Path has svg file extension");
            (args.register_icon_texture_id)(context, &ArcStr::from(name), texture_id);
        } else if path.join("model.toml").exists() {
            log::debug!("Rendering SVG model {}", path.display());

            let model = fs::read_to_string(path.join("model.toml"))
                .with_context(|| format!("Reading {}/model.toml", path.display()))?;
            let model: ModelToml = toml::from_str(&model)
                .with_context(|| format!("Parsing {}/model.toml", path.display()))?;

            let dimension = model.unit.spritesheet_side() * max_dim;
            let mut pixmap = Pixmap::new(dimension, dimension).context("Creating pixmap buffer")?;
            pixmap.fill(tiny_skia::Color::WHITE);

            for (sprite_id, file) in model.unit.sprite_names().iter().enumerate() {
                let svg_path = path.join(file).with_extension("svg");
                let sprite_pixmap = render_svg(
                    &svg_path,
                    max_dim,
                    &options,
                    args.skip_svg,
                    args.render_timer.start(),
                )
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

            if !args.skip_svg {
                let args = WriteVariantsArgs::builder()
                    .variants(args.atlas.variants())
                    .pixmap(pixmap)
                    .side_sprite_count(model.unit.spritesheet_side())
                    .pixmap_size(dimension)
                    .output(args.output)
                    .downscale_timer(args.downscale_timer)
                    .save_timer(args.save_timer)
                    .texture_id(texture_id)
                    .build();
                write_variants(args).context("Emitting PNG output")?;
            }

            let name = path.file_name().expect("Path has filename");
            let name = name.to_str().context("Texture name must be ASCII only")?;
            (args.register_model_texture_id)(context, &ArcStr::from(name), texture_id, model.unit);
        }
    }

    Ok(())
}

#[derive(TypedBuilder)]
struct WriteVariantsArgs<'t> {
    variants:          &'t [atlas::Variant],
    pixmap:            Pixmap,
    side_sprite_count: u32,
    pixmap_size:       u32,
    output:            &'t Path,
    downscale_timer:   &'t Timer,
    save_timer:        &'t Timer,
    texture_id:        SpritesheetId,
}

fn write_variants(args: WriteVariantsArgs) -> Result<()> {
    for variant in args.variants {
        let out_png = args.output.join(atlas::to_path(variant.name().as_str(), args.texture_id));
        log::debug!("Saving downscaled variant {}", out_png.display());

        let dir = out_png.parent().expect("File has no parent");
        if !dir.exists() {
            fs::create_dir(&dir)
                .with_context(|| format!("Creating variant directory at {}", dir.display()))?;
        }

        let downscaled_map = {
            let _timer = args.downscale_timer.start();
            downscale(&args.pixmap, args.pixmap_size, variant.dimension() * args.side_sprite_count)
                .context("Downscaling sprite")?
        };
        {
            let _timer = args.save_timer.start();
            downscaled_map
                .save_png(&out_png)
                .with_context(|| format!("Saving atlas as PNG at {}", out_png.display()))?;
        }
    }

    Ok(())
}

fn downscale(map: &Pixmap, old_size: u32, new_size: u32) -> Result<Pixmap> {
    debug_assert!(new_size <= old_size, "new_size ({}) > old_size ({})", new_size, old_size);
    let ratio = new_size.small_float::<f32>() / old_size.small_float::<f32>();
    let side = map.width().small_float::<f32>() * ratio;
    let side = side.trunc_int::<u32>();
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
