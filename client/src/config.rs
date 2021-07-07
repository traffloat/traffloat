//! Hardcoded client constants

/// Moving velocity with WASD per delta time
pub const WASD_LINEAR_VELOCITY: f64 = 0.10;
/// Moving rotation with WASD per delta time
pub const WASD_ROTATION_VELOCITY: f64 = 0.05;
/// Rate of zooming with =- per delta time
pub const ZOOM_VELOCITY: f64 = 0.02;
/// Rate of scrolling per event
pub const SCROLL_VELOCITY: f64 = 0.03;

/// Whether to render debug messages
pub const RENDER_DEBUG: bool = codegen::RENDER_DEBUG;
