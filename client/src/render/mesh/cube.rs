//! A cube model.

use nalgebra::{Vector2, Vector3};
use traffloat::appearance;
use web_sys::WebGlRenderingContext;
use xias::Xias;

/// Creates a cube model.
pub fn prepare(gl: &WebGlRenderingContext) -> impl super::Mesh {
    let mut builder = super::Builder::default();

    for axis in 0..3 {
        for sign in [Sign::Positive, Sign::Negative] {
            let direction = Direction { axis, sign };

            let normal = direction.as_vector();

            let b = {
                let mut vector = Vector3::new(0., 0., 0.);
                vector[(axis + 1) % 3] = sign.as_float();
                vector
            };
            let c = normal.cross(&b);

            let tex_pos = appearance::Unit::Cube
                .search_sprite_coord_by_name(direction.name())
                .expect("Direction::name mismatches Unit::sprite_name");
            let tex_pos = Vector2::new(tex_pos.0.small_float(), tex_pos.1.small_float());

            let corners = [
                Vector2::new(-1., -1.),
                Vector2::new(-1., 1.),
                Vector2::new(1., -1.),
                Vector2::new(1., 1.),
            ];

            for triangle in [[0, 2, 1], [1, 2, 3]] {
                for corner in triangle {
                    let offset = corners[corner];

                    let position = normal + b * offset[0] + c * offset[1];

                    let mut vert_tex_pos = tex_pos + Vector2::new(0.5, 0.5) + offset * 0.5;
                    vert_tex_pos /= 4.; // 6 sprites for a cube mesh, fits on a 4^2 spritesheet.
                    builder.push(position, normal, vert_tex_pos);
                }
            }
        }
    }

    builder.compile_unindexed(gl)
}

#[derive(Debug, Clone, Copy)]
struct Direction {
    axis: usize,
    sign: Sign,
}

impl Direction {
    fn as_vector(self) -> Vector3<f32> {
        let mut triple = [0.; 3];
        triple[self.axis] = self.sign.as_float();
        Vector3::from_iterator(triple)
    }

    fn name(&self) -> &'static str {
        match self {
            Self { axis: 0, sign: Sign::Positive } => "xp",
            Self { axis: 0, sign: Sign::Negative } => "xn",
            Self { axis: 1, sign: Sign::Positive } => "yp",
            Self { axis: 1, sign: Sign::Negative } => "yn",
            Self { axis: 2, sign: Sign::Positive } => "zp",
            Self { axis: 2, sign: Sign::Negative } => "zn",
            _ => unreachable!("Constructed direction with invalid axis"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Sign {
    Positive,
    Negative,
}

impl Sign {
    fn as_float(self) -> f32 {
        match self {
            Self::Positive => 1.,
            Self::Negative => -1.,
        }
    }
}
