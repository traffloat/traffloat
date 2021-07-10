use std::marker::PhantomData;

use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlUniformLocation};

/// Stores a uniform location with known type.
pub struct UniformLocation<T> {
    _ph: PhantomData<*mut T>,
    loc: WebGlUniformLocation,
}

impl<T: Uniform> UniformLocation<T> {
    /// Locates a uniform for a given program.
    pub fn new(gl: &WebGlRenderingContext, program: &WebGlProgram, name: &str) -> Self {
        let loc = match gl.get_uniform_location(program, name) {
            Some(loc) => loc,
            None => panic!("Uniform {:?} does not exist in program", name),
        };
        Self {
            _ph: PhantomData,
            loc,
        }
    }

    /// Assigns a value for the uniform.
    pub fn assign(&self, gl: &WebGlRenderingContext, value: T) {
        value.apply(Some(&self.loc), gl);
    }
}

/// A type that can be assigned as a uniform.
pub trait Uniform {
    /// Assigns a value for hte uniform.
    fn apply(&self, location: Option<&WebGlUniformLocation>, gl: &WebGlRenderingContext);
}

macro_rules! impl_uniform {
    ($unif:ident, $vec:ty, $method:ident, {$($extra:tt)*}) => {
        impl Uniform for $vec {
            fn apply(&self, location: Option<&WebGlUniformLocation>, gl: &WebGlRenderingContext) {
                gl.$unif(location, $($extra)* self.$method());
            }
        }
    }
}

impl_uniform!(
    uniform2fv_with_f32_array,
    nalgebra::Vector2<f32>,
    as_slice,
    {}
);
impl_uniform!(
    uniform3fv_with_f32_array,
    nalgebra::Vector3<f32>,
    as_slice,
    {}
);
impl_uniform!(
    uniform4fv_with_f32_array,
    nalgebra::Vector4<f32>,
    as_slice,
    {}
);

impl_uniform!(uniform2fv_with_f32_array, [f32; 2], as_ref, {});
impl_uniform!(uniform3fv_with_f32_array, [f32; 3], as_ref, {});
impl_uniform!(uniform4fv_with_f32_array, [f32; 4], as_ref, {});

impl_uniform!(uniform_matrix2fv_with_f32_array, nalgebra::Matrix2<f32>, as_slice, {false, });
impl_uniform!(uniform_matrix3fv_with_f32_array, nalgebra::Matrix3<f32>, as_slice, {false, });
impl_uniform!(uniform_matrix4fv_with_f32_array, nalgebra::Matrix4<f32>, as_slice, {false, });

impl Uniform for f32 {
    fn apply(&self, location: Option<&WebGlUniformLocation>, gl: &WebGlRenderingContext) {
        gl.uniform1f(location, *self);
    }
}
