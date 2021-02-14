use crate::render::util::{FloatBuffer, IndexBuffer};

/// A complex object to render
pub struct Mesh {
    positions: FloatBuffer,
    faces: IndexBuffer,
}
