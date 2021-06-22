use typed_builder::TypedBuilder;

use crate::render::util;

/// A complex object to render
#[derive(getset::Getters, TypedBuilder)]
pub struct Mesh {
    /// The position buffer.
    #[getset(get = "pub")]
    positions: util::FloatBuffer,
    /// The face index buffer.
    #[getset(get = "pub")]
    faces: util::IndexBuffer,
}
