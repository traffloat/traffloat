use std::mem::size_of;

#[cfg(not(target_endian = "little"))]
compile_error!("Only little endian archs are supported");

#[cfg(feature = "enc")]
mod enc;
#[cfg(feature = "enc")]
pub use enc::*;

#[cfg(feature = "dec")]
mod dec;
#[cfg(feature = "dec")]
pub use dec::*;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex(pub [f32; 3]);

pub type FaceIndex = i16;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Face(pub [FaceIndex; 3]);
