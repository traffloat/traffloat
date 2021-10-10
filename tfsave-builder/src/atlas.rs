use std::ffi::OsStr;
use std::path::Path;
use std::{fs, mem};

use anyhow::{Context, Result};
use safety::Safety;
use serde::Deserialize;
use tiny_skia::{Pixmap, PixmapPaint, PixmapRef};
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
) -> Result<()> {
    let input = input.canonicalize().context("Cannot canonicalize input file")?;
    let dir = input.parent().context("Regular file has no parent")?;
    let mut dir = dir.join(def.dir().as_str());
    dir = dir
        .canonicalize()
        .with_context(|| format!("Cannont canonicalize atlas path {}", dir.display()))?;

    log::info!("Generating atlas for {}", dir.display());

    let options = usvg::Options::default();
    let options = options.to_ref();

    let max_dim = 1024;
    let mut atlas = Atlas::new(max_dim)?;

    for file in fs::read_dir(&dir).context("Scanning directory list")? {
        let file = file.context("Scanning directory list")?;
        let path = file.path();

        if path.extension() == Some(OsStr::new("svg")) {
            log::debug!("Rendering SVG file {}", path.display());

            let pixmap = render_svg(&path, max_dim, &options, render_timer.start())
                .with_context(|| format!("Rendering SVG file {}", path.display()))?;
            atlas.add(pixmap.as_ref())?;
        } else if path.join("model.toml").exists() {
            log::debug!("Rendering SVG model {}", path.display());
            let model = fs::read_to_string(path.join("model.toml"))
                .with_context(|| format!("Reading {}/model.toml", path.display()))?;
            let model: ModelToml = toml::from_str(&model)
                .with_context(|| format!("Parsing {}/model.toml", path.display()))?;

            let files = match model.unit {
                geometry::Unit::Cube => vec!["xp", "xn", "yp", "yn", "zp", "zn"],
                geometry::Unit::Cylinder => vec!["top", "bottom", "curved"],
                geometry::Unit::Sphere => unimplemented!("Spheres are not supported yet"),
            };

            for file in files {
                let svg_path = path.join(file).with_extension("svg");
                let pixmap = render_svg(&svg_path, max_dim, &options, render_timer.start())
                    .with_context(|| format!("Rendering SVG file {}", svg_path.display()))?;
                atlas.add(pixmap.as_ref()).context("Adding SVG file to atlas")?;
            }
        }
    }

    for variant in def.variants() {
        let out_png = output.join(format!("{}-{}.png", def.res_name(), variant.name()));
        log::debug!("Saving downscaled variant {}", out_png.display());
        let downscaled_map = {
            let _timer = downscale_timer.start();
            atlas.downscale(variant.dimension()).context("Downscaling atlas")?
        };
        {
            let _timer = save_timer.start();
            downscaled_map.save_png(out_png).context("Saving atlas as PNG")?;
        }
    }

    Ok(())
}

struct Atlas {
    map:            Pixmap,
    item_size:      u32,
    items_per_axis: u32,
    counter:        u32,
}

impl Atlas {
    fn new(size: u32) -> Result<Self> {
        Ok(Self {
            map:            Pixmap::new(size, size).context("Creating pixmap buffer")?,
            item_size:      size,
            items_per_axis: 1,
            counter:        0,
        })
    }

    fn copy(&mut self, x: i32, y: i32, map: PixmapRef) -> Result<()> {
        self.map
            .draw_pixmap(x, y, map, &PixmapPaint::default(), tiny_skia::Transform::identity(), None)
            .context("Drawing onto output canvas")
    }

    fn add(&mut self, map: PixmapRef) -> Result<()> {
        if self.counter == self.items_per_axis.pow(2) {
            self.items_per_axis *= 2;
            let dim = self.item_size * self.items_per_axis;
            let old_map = mem::replace(
                &mut self.map,
                Pixmap::new(dim, dim).context("Enlarging pixmap buffer")?,
            );

            self.copy(0, 0, old_map.as_ref())?;
        }

        let (x, y) = locate_xy(self.items_per_axis, self.counter);
        self.counter += 1;
        self.copy((x * self.item_size).homosign(), (y * self.item_size).homosign(), map)?;

        Ok(())
    }

    fn downscale(&self, size: u32) -> Result<Pixmap> {
        assert!(size <= self.item_size);
        let ratio = size.small_float() / self.item_size.small_float();
        let side = size * self.items_per_axis;
        let mut map = Pixmap::new(side, side).context("Creating downscaled pixmap buffer")?;
        map.draw_pixmap(
            0,
            0,
            self.map.as_ref(),
            &PixmapPaint::default(),
            tiny_skia::Transform::from_scale(ratio, ratio),
            None,
        )
        .context("Downscaling pixmap buffer")?;
        Ok(map)
    }
}

fn locate_xy(side: u32, count: u32) -> (u32, u32) {
    if count == 0 {
        return (0, 0);
    }
    let subarea = side * side / 4;
    let (x, y) = locate_xy(side / 2, count % subarea);
    match count / subarea {
        0 => (x, y),
        1 => (x, y + side / 2),
        2 => (x + side / 2, y),
        3 => (x + side / 2, y + side / 2),
        _ => unreachable!("count > side * side"),
    }
}

fn render_svg(
    path: &Path,
    dimension: u32,
    options: &usvg::OptionsRef,
    _timer: TimerStart<'_>,
) -> Result<Pixmap> {
    let data = fs::read(path).context("Reading SVG")?;

    let tree = usvg::Tree::from_data(&data, options).context("Parsing SVG")?;
    let mut pixmap = Pixmap::new(dimension, dimension).context("Creating pixmap buffer")?;

    resvg::render(&tree, usvg::FitTo::Size(dimension, dimension), pixmap.as_mut())
        .context("Rendering SVG")?;

    Ok(pixmap)
}

#[derive(Deserialize)]
struct ModelToml {
    unit: geometry::Unit,
}
