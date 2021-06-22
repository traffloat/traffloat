use std::cell::RefCell;
use std::rc::Rc;

use web_sys::{CanvasRenderingContext2d, ImageBitmap, WebGlRenderingContext};

use crate::util::DebugWriter;
use traffloat::space::Matrix;

/// The dimension of a canvas
#[derive(Debug, Clone, Copy)]
pub struct Dimension {
    /// Number of pixels horizontally.
    pub width: u32,
    /// Number of pixels vertically.
    pub height: u32,
}

impl Dimension {
    /// Aspect ratio of the dimension
    pub fn aspect(self) -> f64 {
        (self.width as f64) / (self.height as f64)
    }
}

/// A shared reference to a canvas.
pub type Canvas = Rc<RefCell<CanvasStruct>>;

/// Information for the canvas.
///
/// This stores three underlying canvas,
/// namely background, scene and UI.
pub struct CanvasStruct {
    bg: super::bg::Setup,
    scene: super::scene::Setup,
    ui: web_sys::CanvasRenderingContext2d,
    debug: super::debug::Setup,
}

impl CanvasStruct {
    /// Instantiates the canvas wrapper.
    pub fn new(
        bg: WebGlRenderingContext,
        scene: WebGlRenderingContext,
        ui: CanvasRenderingContext2d,
        debug: DebugWriter,
    ) -> Canvas {
        let bg = super::bg::setup(bg);
        let scene = super::scene::setup(scene);
        let debug = super::debug::Setup::new(debug);

        Rc::new(RefCell::new(Self {
            bg,
            scene,
            ui,
            debug,
        }))
    }

    /// Resets to the rendering to a new frame.
    pub fn new_frame(&mut self, dim: &Dimension) {
        self.scene.clear();

        self.ui
            .reset_transform()
            .expect("CanvasRenderingContext2d.resetTransform() threw");
        self.ui
            .clear_rect(0., 0., dim.width as f64, dim.height as f64);
        self.ui.set_stroke_style(&"black".into());
        self.ui.set_fill_style(&"white".into());
        self.ui.set_font("12px sans-serif");

        self.debug.reset();
    }

    /// Draws the background.
    pub fn draw_bg(&self, rot: Matrix, aspect: f32) {
        self.bg.reset();

        // TODO draw stars

        self.bg.draw_sun(rot, aspect);
    }

    /// Draws an object at the given transformation from shape coordinates to world coordinates.
    pub fn draw_object(&self, proj: Matrix) {
        self.scene.draw_object(proj);
    }

    /// Retrieves the debug layer.
    pub fn debug(&self) -> &super::debug::Setup {
        &self.debug
    }
}

/// Provides an [`ImageBitmap`][ImageBitmap].
pub trait Image {
    /// Converts the value into an [`ImageBitmap`][ImageBitmap].
    fn as_bitmap(&self) -> Option<&ImageBitmap>;
}
