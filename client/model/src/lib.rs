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

pub type FaceIndex = u16;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Face(pub [FaceIndex; 3]);

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Color(pub [f32; 3]);

/// Transmutes a slice from type T to type U
///
/// # Safety
/// The sizes of `T` and `U` must be multiple of one another,
/// such that the representation of sufficiently many continguous units of `T` represents
/// contiguous units of `U`.
///
/// Furthermore, it is assumed that the size of the input slice is a multiple of this ratio.
pub unsafe fn transmute_slice<T, U>(slice: &[T]) -> &[U] {
    let size = slice.len() * size_of::<T>() / size_of::<U>();
    std::slice::from_raw_parts(slice.as_ptr() as *const U, size)
}
