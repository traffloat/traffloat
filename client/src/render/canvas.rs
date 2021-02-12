use web_sys::{CanvasRenderingContext2d, WebGlRenderingContext};

use traffloat::space::{Matrix, Vector};

/// The dimension of a canvas
#[derive(Debug, Clone, Copy)]
pub struct Dimension {
    pub width: u32,
    pub height: u32,
}

impl Dimension {
    /// Aspect ratio of the dimension
    pub fn aspect(self) -> f64 {
        (self.width as f64) / (self.height as f64)
    }
}

/// Information for a canvas
pub struct Canvas {
    pub bg: web_sys::WebGlRenderingContext,
    pub scene: web_sys::WebGlRenderingContext,
    pub ui: web_sys::CanvasRenderingContext2d,
    debug_count: u32,
}

impl Canvas {
    pub fn new(
        bg: WebGlRenderingContext,
        scene: WebGlRenderingContext,
        ui: CanvasRenderingContext2d,
    ) -> Self {
        Self {
            bg,
            scene,
            ui,
            debug_count: 0,
        }
    }

    pub fn new_frame(&mut self, dim: &Dimension) {
        self.scene.clear_color(0., 0., 0., 0.);

        self.ui
            .reset_transform()
            .expect("CanvasRenderingContext2d.resetTransform() threw");
        self.ui
            .clear_rect(0., 0., dim.width as f64, dim.height as f64);
        self.ui.set_stroke_style(&"black".into());
        self.ui.set_fill_style(&"white".into());
        self.ui.set_font("12px sans-serif");

        self.debug_count = 0;
    }

    pub fn write_debug(&mut self, line: impl AsRef<str>) {
        self.ui
            .stroke_text(line.as_ref(), 10., 20. + (self.debug_count as f64) * 15.)
            .expect("Failed to draw debug text");
        self.ui
            .fill_text(line.as_ref(), 10., 20. + (self.debug_count as f64) * 15.)
            .expect("Failed to draw debug text");

        self.debug_count += 1;
    }

    pub fn draw_bg(&self, yaw: f64, pitch: f64, roll: f64) {
        self.bg.clear_color(0., 0., 0., 1.);
        self.bg.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        self.scene.clear_color(0., 0., 0., 0.);
        self.scene.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        // TODO draw stars
    }
}

pub trait Image {
    fn as_bitmap(&self) -> Option<&web_sys::ImageBitmap>;
}
