use web_sys::{WebGlRenderingContext, WebGlUniformLocation};

use super::Uniform;
use safety::Safety;

pub trait Glize: Sized {
    type Output: Sized;

    fn glize(self) -> Self::Output;
}

macro_rules! impl_glize {
    ($ident:ident) => {
        impl Glize for nalgebra::$ident<f64> {
            type Output = nalgebra::$ident<f32>;

            fn glize(self) -> Self::Output {
                nalgebra::$ident::from_iterator(self.iter().map(|&f| f.lossy_trunc()))
            }
        }
    };
}

impl Glize for f64 {
    type Output = f32;

    fn glize(self) -> Self::Output {
        self.lossy_trunc()
    }
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
