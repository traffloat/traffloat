use std::cell::RefCell;
use std::rc::Rc;

use web_sys::{CanvasRenderingContext2d, WebGlRenderingContext};

use super::{bg, debug, scene, ui};
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
pub type Layers = Rc<RefCell<LayersStruct>>;

/// Information for the canvas.
///
/// This stores three underlying canvas,
/// namely background, scene and UI.
#[derive(getset::Getters, getset::MutGetters)]
pub struct LayersStruct {
    /// The background render layer
    #[getset(get = "pub")]
    bg: bg::Canvas,
    /// The object render layer
    #[getset(get = "pub")]
    scene: scene::Canvas,
    /// The UI 2D canvas layer
    #[getset(get = "pub")]
    ui: ui::Canvas,
    /// The debug DOM layer
    #[getset(get = "pub", get_mut = "pub")]
    debug: debug::Canvas,
}

impl LayersStruct {
    /// Instantiates the canvas wrapper.
    pub fn new(
        bg: WebGlRenderingContext,
        scene: WebGlRenderingContext,
        ui: CanvasRenderingContext2d,
        debug: DebugWriter,
    ) -> Layers {
        let bg = bg::Canvas::new(bg);
        let scene = scene::Canvas::new(scene);
        let ui = ui::Canvas::new(ui);
        let debug = debug::Canvas::new(debug);

        Rc::new(RefCell::new(Self {
            bg,
            scene,
            ui,
            debug,
        }))
    }
}
