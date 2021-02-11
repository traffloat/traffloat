use traffloat::types::{Matrix, Vector};

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
        bg: web_sys::WebGlRenderingContext,
        scene: web_sys::WebGlRenderingContext,
        ui: web_sys::CanvasRenderingContext2d,
    ) -> Self {
        ui.reset_transform()
            .expect("CanvasRenderingContext2d.resetTransform() threw");
        ui.set_stroke_style(&"black".into());
        ui.set_fill_style(&"white".into());
        ui.set_font("12px sans-serif");
        Self {
            bg,
            scene,
            ui,
            debug_count: 0,
        }
    }

    pub fn new_frame(&mut self) {
        self.scene.clear_color(0., 0., 0., 0.);
        self.debug_count = 0;
    }

    pub fn write_debug(&mut self, line: impl AsRef<str>) {
        self.ui
            .stroke_text(line.as_ref(), 10., 20. + (self.debug_count as f64) * 15.)
            .expect("Failed to draw debug text");
        self.ui
            .fill_text(line.as_ref(), 10., 20. + (self.debug_count as f64) * 15.)
            .expect("Failed to draw debug text");
    }

    pub fn draw_bg(&self, yaw: f64, pitch: f64, roll: f64) {
        // TODO
    }
}

pub trait Image {
    fn as_bitmap(&self) -> Option<&web_sys::ImageBitmap>;
}
