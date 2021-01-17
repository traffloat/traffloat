use std::fmt;
use std::slice;

use super::*;

#[cfg(target_endian = "big")]
compile_error!("Unsupported architecture: big-endian byte order");

#[cfg(target_endian = "little")]
pub fn decode_vertices(slice: &[u8]) -> &[f32] {
    let len = slice.len();
    #[allow(clippy::size_of_in_element_count)]
    unsafe {
        slice::from_raw_parts(slice.as_ptr() as *const f32, len / size_of::<f32>())
    }
}

#[cfg(target_endian = "little")]
pub fn decode_normals(slice: &[u8]) -> &[f32] {
    let len = slice.len();
    #[allow(clippy::size_of_in_element_count)]
    unsafe {
        slice::from_raw_parts(slice.as_ptr() as *const f32, len / size_of::<f32>())
    }
}

#[cfg(target_endian = "little")]
pub fn decode_faces(slice: &[u8]) -> &[FaceIndex] {
    let len = slice.len();
    #[allow(clippy::size_of_in_element_count)]
    unsafe {
        slice::from_raw_parts(
            slice.as_ptr() as *const FaceIndex,
            len / size_of::<FaceIndex>(),
        )
    }
}

#[cfg(target_endian = "little")]
pub fn decode_colors(slice: &[u8]) -> &[f32] {
    let len = slice.len();
    #[allow(clippy::size_of_in_element_count)]
    unsafe {
        slice::from_raw_parts(slice.as_ptr() as *const f32, len / size_of::<f32>())
    }
}

#[derive(Clone, Copy)]
pub struct RawMesh {
    pub vertices: &'static [u8],
    pub normals: &'static [u8],
    pub colors: &'static [u8],
    pub faces: &'static [u8],
}

pub trait AbstractMesh {
    fn vertices(&self) -> &[f32];
    fn normals(&self) -> &[f32];
    fn colors(&self) -> &[f32];
    fn faces(&self) -> &[FaceIndex];
}

impl AbstractMesh for RawMesh {
    fn vertices(&self) -> &[f32] {
        decode_vertices(self.vertices)
    }

    fn normals(&self) -> &[f32] {
        decode_normals(self.normals)
    }

    fn colors(&self) -> &[f32] {
        decode_colors(self.colors)
    }

    fn faces(&self) -> &[FaceIndex] {
        decode_faces(self.faces)
    }
}

pub struct DynamicMesh {
    pub vertices: Vec<Vertex>,
    pub normals: Vec<Normal>,
    pub colors: Vec<Color>,
    pub faces: Vec<Face>,
}

impl AbstractMesh for DynamicMesh {
    fn vertices(&self) -> &[f32] {
        let ptr = self.vertices.as_ptr() as *const f32;
        unsafe { slice::from_raw_parts(ptr, self.vertices.len() * 3) }
    }

    fn normals(&self) -> &[f32] {
        let ptr = self.normals.as_ptr() as *const f32;
        unsafe { slice::from_raw_parts(ptr, self.normals.len() * 3) }
    }

    fn colors(&self) -> &[f32] {
        let ptr = self.colors.as_ptr() as *const f32;
        unsafe { slice::from_raw_parts(ptr, self.colors.len() * 3) }
    }

    fn faces(&self) -> &[FaceIndex] {
        let ptr = self.faces.as_ptr() as *const FaceIndex;
        unsafe { slice::from_raw_parts(ptr, self.faces.len() * 3) }
    }
}

impl fmt::Debug for RawMesh {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v: Vec<_> = self.vertices().chunks(3).collect();
        f.debug_list()
            .entries(self.faces().chunks(3).map(|indices| {
                [
                    v[indices[0] as usize],
                    v[indices[1] as usize],
                    v[indices[2] as usize],
                ]
            }))
            .finish()
    }
}
impl fmt::Debug for DynamicMesh {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.faces.iter().map(|Face(indices)| {
                [
                    self.vertices[indices[0] as usize],
                    self.vertices[indices[1] as usize],
                    self.vertices[indices[2] as usize],
                ]
            }))
            .finish()
    }
}
