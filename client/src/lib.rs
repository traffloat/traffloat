#![feature(div_duration)]

mod input;
mod interface;
mod state;
mod windowing;

pub use interface::*;
pub use state::*;
pub use windowing::*;
use xias::Xias;

fn vec(v: traffloat_types::space::Vector) -> three_d::Vec3 {
    three_d::Vec3::new(v.x.lossy_float(), v.y.lossy_float(), v.z.lossy_float())
}

fn mat(m: traffloat_types::space::Matrix) -> three_d::Mat4 {
    three_d::Mat4::new(
        m[(0, 0)].lossy_float(),
        m[(0, 1)].lossy_float(),
        m[(0, 2)].lossy_float(),
        m[(0, 3)].lossy_float(),
        m[(1, 0)].lossy_float(),
        m[(1, 1)].lossy_float(),
        m[(1, 2)].lossy_float(),
        m[(1, 3)].lossy_float(),
        m[(2, 0)].lossy_float(),
        m[(2, 1)].lossy_float(),
        m[(2, 2)].lossy_float(),
        m[(2, 3)].lossy_float(),
        m[(3, 0)].lossy_float(),
        m[(3, 1)].lossy_float(),
        m[(3, 2)].lossy_float(),
        m[(3, 3)].lossy_float(),
    )
}
