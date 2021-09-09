use std::marker::PhantomData;

use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlUniformLocation};

/// Stores a uniform location with known type.
pub struct UniformLocation<T> {
    _ph: PhantomData<*mut T>,
    loc: Option<WebGlUniformLocation>,
}

impl<T: Uniform> UniformLocation<T> {
    /// Locates a uniform for a given program.
    pub fn new(gl: &WebGlRenderingContext, program: &WebGlProgram, name: &str) -> Self {
        let ret = Self::new_optional(gl, program, name);
        if ret.loc.is_none() {
            panic!("Uniform {:?} does not exist in program", name);
        }
        ret
    }

    /// Locates a uniform for a given program.
    pub fn new_optional(gl: &WebGlRenderingContext, program: &WebGlProgram, name: &str) -> Self {
        let loc = gl.get_uniform_location(program, name);
        Self { _ph: PhantomData, loc }
    }

    /// Assigns a value for the uniform.
    pub fn assign(&self, gl: &WebGlRenderingContext, value: T) {
        value.apply(self.loc.as_ref(), gl);
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

impl_uniform!(uniform2fv_with_f32_array, nalgebra::Vector2<f32>, as_slice, {});
impl_uniform!(uniform3fv_with_f32_array, nalgebra::Vector3<f32>, as_slice, {});
impl_uniform!(uniform4fv_with_f32_array, nalgebra::Vector4<f32>, as_slice, {});

impl_uniform!(uniform2fv_with_f32_array, [f32; 2], as_ref, {});
impl_uniform!(uniform3fv_with_f32_array, [f32; 3], as_ref, {});
impl_uniform!(uniform4fv_with_f32_array, [f32; 4], as_ref, {});

impl_uniform!(uniform2iv_with_i32_array, [i32; 2], as_ref, {});
impl_uniform!(uniform3iv_with_i32_array, [i32; 3], as_ref, {});
impl_uniform!(uniform4iv_with_i32_array, [i32; 4], as_ref, {});

impl_uniform!(uniform_matrix2fv_with_f32_array, nalgebra::Matrix2<f32>, as_slice, {false, });
impl_uniform!(uniform_matrix3fv_with_f32_array, nalgebra::Matrix3<f32>, as_slice, {false, });
impl_uniform!(uniform_matrix4fv_with_f32_array, nalgebra::Matrix4<f32>, as_slice, {false, });

impl Uniform for f32 {
    fn apply(&self, location: Option<&WebGlUniformLocation>, gl: &WebGlRenderingContext) {
        gl.uniform1f(location, *self);
    }
}

impl Uniform for i32 {
    fn apply(&self, location: Option<&WebGlUniformLocation>, gl: &WebGlRenderingContext) {
        gl.uniform1i(location, *self);
    }
}
