//! Shape and appearance of an object

use derive_new::new;
use smallvec::{smallvec, SmallVec};
use typed_builder::TypedBuilder;

use crate::config::{self, Config};
use crate::space::{Matrix, Point, Position, Vector};
use crate::SetupEcs;

/// Describes the shape and appearance of an object
#[derive(TypedBuilder, getset::CopyGetters)]
pub struct Shape {
    #[getset(get_copy = "pub")]
    /// Unit shape variant
    unit: Unit,
    /// The transformation matrix from the unit shape to this shape centered at the origin.
    #[getset(get_copy = "pub")]
    matrix: Matrix,
    /// The inverse transformation matrix from this shape centered at the origin to the unit shape.
    #[getset(get_copy = "pub")]
    #[builder(
        default_code = r#"matrix.try_inverse().expect("Transformation matrix is singular")"#
    )]
    inv_matrix: Matrix,
    /// The texture for rendering the shape
    #[getset(get_copy = "pub")]
    texture: config::Id<Texture>,
}

impl Shape {
    /// The transformation matrix from the unit shape to this shape centered at pos
    pub fn transform(&self, pos: Position) -> Matrix {
        self.matrix().append_translation(&pos.vector())
    }

    /// The transformation matrix from this shape centered at pos to the unit shape
    pub fn inv_transform(&self, pos: Position) -> Matrix {
        self.inv_matrix().prepend_translation(&-pos.vector())
    }
}

/// A unit shape variant
#[derive(Debug, Clone, Copy)]
pub enum Unit {
    /// A unit cube `[-1, 1]^3`
    Cube,
    /// A unit sphere `x^2 + y^2 + z^2 <= 1`
    Sphere,
}

impl Unit {
    /// Checks whether the given point is within this unit shape
    pub fn contains(&self, pos: Point) -> bool {
        match self {
            Self::Cube => {
                (-1. ..=1.).contains(&pos.x)
                    && (-1. ..=1.).contains(&pos.y)
                    && (-1. ..=1.).contains(&pos.z)
            }
            Self::Sphere => pos.x.powi(2) + pos.y.powi(2) + pos.z.powi(2) <= 1.,
        }
    }

    /// Checks whether the line segment between `start` and `end` intersects with this unit shape.
    ///
    /// If it does, returns a weight `w` (`0 <= w <= 1`) at which `start * (1 - w) + end * w` is
    /// within this shape. It is unspecified which particular point within the intersecting segment
    /// is returned.
    pub fn between(&self, start: Point, end: Point) -> Option<f64> {
        match self {
            Self::Cube => {
                let dir = end - start;

                for dim in 0..3 {
                    #[allow(clippy::indexing_slicing)]
                    for &target in &[-1., 1.] {
                        let w = (target - start[dim]) / dir[dim];
                        if (0. ..=1.).contains(&w) {
                            let point = start + dir * w;
                            let inside = (0..3)
                                .filter(|&other| other != dim)
                                .all(|other| (-1. ..=1.).contains(&point[other]));
                            if inside {
                                return Some(w);
                            }
                        }
                    }
                }

                None
            }
            Self::Sphere => {
                if self.contains(start) {
                    return Some(0.);
                }
                if self.contains(end) {
                    return Some(1.);
                }

                let dir = end - start;

                // Neither endpoint is within the sphere.
                // If the sphere contains part of the segment, the closest point of the line
                // containing the segment from the sphere center must also be within the segment.
                let w = (Point::origin() - start).dot(&dir) / dir.norm_squared();
                let closest = start + dir * w;
                self.contains(closest).then(|| w)
            }
        }
    }

    /// Computes the axis-aligned bounding box under the given transformation matrix
    ///
    /// The transformation matrix should transform the unit shape to the real coordinates.
    pub fn bb_under(&self, transform: Matrix) -> (Point, Point) {
        use nalgebra::dimension as dim;

        match self {
            Self::Cube => {
                type Storage = nalgebra::storage::Owned<f64, dim::U4, dim::U8>;
                type Points = nalgebra::Matrix<f64, dim::U4, dim::U8, Storage>;

                fn p01() -> impl Iterator<Item = f64> {
                    [0., 1.].iter().copied()
                }
                fn xyz(x: f64, y: f64, z: f64) -> impl Iterator<Item = f64> {
                    let vec: SmallVec<[f64; 4]> = smallvec![x, y, z, 1.];
                    vec.into_iter()
                }
                let iter = p01()
                    .flat_map(|x| p01().flat_map(move |y| p01().flat_map(move |z| xyz(x, y, z))));
                let mut points = Points::from_iterator(iter);
                points = transform * points;

                let min: SmallVec<[f64; 3]> = (0_usize..3).map(|i| points.row(i).min()).collect();
                let max: SmallVec<[f64; 3]> = (0_usize..3).map(|i| points.row(i).max()).collect();

                #[allow(clippy::indexing_slicing)]
                (
                    Point::new(min[0], min[1], min[2]),
                    Point::new(max[0], max[1], max[2]),
                )
            }
            Self::Sphere => {
                // Extremize f(x,y,z) := ax+by+xz+d under g(x,y,z) := x^2+y^2+z^2-1 = 0
                // By Lagrange multipliers theorem,
                // solving d/d[xyz] f(x,y,z) = lambda * d/d[xyz] g(x,y,z)
                // gives the following equations for abc != 0:
                // x = \pm a / sqrt(a^2+b^2+c^2)
                // y = \pm b / sqrt(a^2+b^2+c^2)
                // z = \pm c / sqrt(a^2+b^2+c^2)

                #[allow(clippy::indexing_slicing)]
                let extrema: SmallVec<[(f64, f64); 3]> = (0_usize..3)
                    .map(|i| {
                        let row = transform.row(i);

                        let norm = row.fixed_slice::<1, 3>(0, 0).norm();

                        let points: SmallVec<[f64; 2]> = [-1_f64, 1.]
                            .iter()
                            .map(|&sgn| {
                                let unit = Vector::from_iterator(
                                    (0_usize..3).map(|j| sgn * row[j] / norm),
                                )
                                .fixed_resize::<4, 1>(1.);
                                (row * unit)[0]
                            })
                            .collect();
                        (points[0], points[1])
                    })
                    .collect();

                let min = Point::from(Vector::from_iterator(
                    extrema.iter().map(|&(i, j)| i.min(j)),
                ));
                let max = Point::from(Vector::from_iterator(
                    extrema.iter().map(|&(i, j)| i.max(j)),
                ));

                (min, max)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Unit;
    use crate::space::{Matrix, Vector};

    #[test]
    pub fn sphere_bb() {
        macro_rules! assert_pt {
            ($pt:expr, ($x:expr, $y:expr, $z:expr)) => {
                let a = &$pt.coords;
                let b = &Vector::new($x, $y, $z);
                let delta = (a - b).norm();
                if delta > 1e-10 {
                    panic!("{} != {}", a, b);
                }
            };
        }
        macro_rules! assert_bb {
            ($trans:expr, ($x0:expr, $y0:expr, $z0:expr)..($x1:expr, $y1:expr, $z1:expr)) => {{
                // type coercion
                fn trans() -> impl FnOnce(Matrix) -> Matrix {
                    $trans
                }
                let trans = trans();
                let mut m = Matrix::identity();
                m = trans(m);
                let bb = Unit::Sphere.bb_under(m);
                assert_pt!(bb.0, ($x0, $y0, $z0));
                assert_pt!(bb.1, ($x1, $y1, $z1));
            }};
        }

        assert_bb!(|m| m, (-1., -1., -1.)..(1., 1., 1.));
        assert_bb!(
            |m| m.append_translation(&Vector::new(0.5, 0.5, 0.5)),
            (-0.5, -0.5, -0.5)..(1.5, 1.5, 1.5)
        );
        assert_bb!(
            |m| m.append_nonuniform_scaling(&Vector::new(0.5, 2., 5.)),
            (-0.5, -2., -5.)..(0.5, 2., 5.)
        );

        {
            assert_bb!(
                |m| {
                    use std::f64::consts::PI;
                    let rot = nalgebra::Rotation3::from_axis_angle(&Vector::x_axis(), PI / 2.);
                    rot.matrix().to_homogeneous() * m.append_translation(&Vector::new(1., 1., 1.))
                },
                (0., -2., 0.)..(2., 0., 2.)
            );
        }
    }
}

/// The texture of a rendered object
#[derive(Debug, new, getset::Getters)]
pub struct Texture {
    /// A URL to an image file
    #[getset(get = "pub")]
    url: String,
    /// The name of the texture.
    #[getset(get = "pub")]
    name: String,
}

impl Config for Texture {}

/// Initializes systems
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
}
