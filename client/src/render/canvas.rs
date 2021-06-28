use std::cell::RefCell;
use std::rc::Rc;

use web_sys::{CanvasRenderingContext2d, WebGlRenderingContext};

use crate::util::DebugWriter;

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
#[derive(getset::Getters, getset::MutGetters)]
pub struct CanvasStruct {
    /// The background render layer
    #[getset(get = "pub")]
    bg: super::bg::Setup,
    /// The object render layer
    #[getset(get = "pub")]
    scene: super::scene::Setup,
    /// The UI 2D canvas layer
    #[getset(get = "pub")]
    ui: web_sys::CanvasRenderingContext2d,
    /// The debug DOM layer
    #[getset(get = "pub", get_mut = "pub")]
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
}
