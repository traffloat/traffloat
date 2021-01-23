#![allow(clippy::unwrap_used)]

use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader};

pub struct RenderContext {
    gl: WebGlRenderingContext,
    server_seed: u64,
    pub should_render: bool,
}

impl RenderContext {
    pub fn new(gl: WebGlRenderingContext, server_seed: u64) -> Self {
        Self {
            gl,
            server_seed,
            should_render: false,
        }
    }
}

// Safety: everything is Send in wasm
unsafe impl Send for RenderContext {}
// Safety: everything is Sync in wasm
unsafe impl Sync for RenderContext {}

impl specs::Component for RenderContext {
    type Storage = specs::storage::BTreeStorage<Self>;
}
