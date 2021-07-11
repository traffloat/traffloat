//! Hardcoded client constants

use traffloat::time::Time;

/// Moving velocity with WASD per delta time
pub const WASD_LINEAR_VELOCITY: f64 = 0.1;
/// Moving rotation with WASD per delta time
pub const WASD_ROTATION_VELOCITY: f64 = 0.01;
/// Rate of zooming with =- per delta time
pub const ZOOM_VELOCITY: f64 = 0.05;
/// Rate of scrolling per event
pub const SCROLL_VELOCITY: f64 = 0.03;
/// Maximum number of ticks between clicks of a double click.
pub const DOUBLE_CLICK_INTERVAL: Time = Time(50);

/// Whether to render debug messages
pub const RENDER_DEBUG: bool = codegen::RENDER_DEBUG;
