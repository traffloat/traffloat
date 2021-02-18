use typed_builder::TypedBuilder;
use web_sys::{WebGlProgram, WebGlRenderingContext};

use crate::render::util;

/// A complex object to render
#[derive(getset::Getters, TypedBuilder)]
pub struct Mesh {
    #[getset(get = "pub")]
    positions: util::FloatBuffer,
    #[getset(get = "pub")]
    faces: util::IndexBuffer,
}
