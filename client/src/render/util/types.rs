use web_sys::{WebGlRenderingContext, WebGlUniformLocation};
use xias::Xias;

use super::Uniform;

/// A float-based type that can be lossily converted for WebGL compatibility.
pub trait Glize: Sized {
    /// The output type that this type is converted into.
    type Output: Sized;

    /// Lossily convert into the output type.
    fn glize(self) -> Self::Output;
}

macro_rules! impl_glize {
    ($ident:ident) => {
        impl Glize for nalgebra::$ident<f64> {
            type Output = nalgebra::$ident<f32>;

            fn glize(self) -> Self::Output {
                nalgebra::$ident::from_iterator(self.iter().map(|&f| f.lossy_float()))
            }
        }
    };
}

impl Glize for f64 {
    type Output = f32;

    fn glize(self) -> Self::Output { self.lossy_float() }
}

impl_glize!(Matrix2);
impl_glize!(Matrix3);
impl_glize!(Matrix4);

impl_glize!(Vector2);
impl_glize!(Vector3);
impl_glize!(Vector4);

impl<U, T: Copy> Uniform for T
where
    T: Glize<Output = U>,
    U: Uniform,
{
    fn apply(&self, location: Option<&WebGlUniformLocation>, gl: &WebGlRenderingContext) {
        self.glize().apply(location, gl);
    }
}
