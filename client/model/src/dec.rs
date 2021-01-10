use std::fmt;

use super::*;

#[cfg(target_endian = "little")]
pub fn decode_vertices(slice: &[u8]) -> &[f32] {
    let len = slice.len();
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const f32, len / size_of::<f32>()) }
}

#[cfg(target_endian = "little")]
pub fn decode_faces(slice: &[u8]) -> &[FaceIndex] {
    let len = slice.len();
    unsafe {
        std::slice::from_raw_parts(
            slice.as_ptr() as *const FaceIndex,
            len / size_of::<FaceIndex>(),
        )
    }
}

#[derive(Clone, Copy)]
pub struct RawMesh {
    pub vertices: &'static [u8],
    pub faces: &'static [u8],
}

impl RawMesh {
    pub fn vertices(&self) -> &[f32] {
        decode_vertices(self.vertices)
    }

    pub fn faces(&self) -> &[FaceIndex] {
        decode_faces(self.faces)
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
