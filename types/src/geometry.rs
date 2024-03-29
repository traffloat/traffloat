//! Common geometric shapes.

use std::convert::TryInto;
use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};
use smallvec::{smallvec, SmallVec};

use crate::space::{Matrix, Point, Vector};

/// A unit shape variant
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Unit {
    /// A unit cube `[-1, 1]^3`
    Cube,
    /// A unit ball `x^2 + y^2 + z^2 <= 1`
    Sphere,
    /// A unit cylinder `x^2 + y^2 <= 1, 0 <= z <= 1`
    Cylinder,
}

impl Unit {
    /// Returns the names of sprites for model spritesheets for this unit variant.
    ///
    /// Sprite locations should follow this order.
    /// Use [`Unit::sprite_coords`] to get the image coordinates for the shape.
    pub fn sprite_names(&self) -> &'static [&'static str] {
        match self {
            Self::Cube => &["xp", "xn", "yp", "yn", "zp", "zn"],
            Self::Sphere => &["sphere"],
            Self::Cylinder => &["top", "bottom", "curved"],
        }
    }

    /// Returns the coordinates of sprites for model spritesheets for this unit variant.
    ///
    /// `order` is the index of the sprite name in the str slice returned in [`Unit::sprite_names`].
    pub fn sprite_coords(&self, order: u32) -> (u32, u32) {
        let max: u32 = (self.sprite_names().len() - 1).try_into().expect("max is a small number");
        debug_assert!(order <= max);
        sprite_location(order, spritesheet_side(max))
    }

    /// Search the sprite coordinates by name.
    pub fn search_sprite_coord_by_name(&self, name: &str) -> Option<(u32, u32)> {
        let order = self.sprite_names().iter().position(|&item| item == name)?;
        Some(self.sprite_coords(order.try_into().expect("sprite_names is short")))
    }

    /// Number of sprites on each side of the spritesheet.
    pub fn spritesheet_side(&self) -> u32 {
        spritesheet_side(
            (self.sprite_names().len() - 1).try_into().expect("sprite count is a small number"),
        )
    }

    /// Checks whether the given point is within this unit shape
    pub fn contains(&self, pos: Point) -> bool {
        match self {
            Self::Cube => {
                (-1. ..=1.).contains(&pos.x)
                    && (-1. ..=1.).contains(&pos.y)
                    && (-1. ..=1.).contains(&pos.z)
            }
            Self::Sphere => pos.x.powi(2) + pos.y.powi(2) + pos.z.powi(2) <= 1.,
            Self::Cylinder => pos.x.powi(2) + pos.y.powi(2) <= 1. && (0. ..=1.).contains(&pos.z),
        }
    }

    /// Checks whether the line segment between `start` and `end` intersects with this unit shape.
    ///
    /// If it does, returns the smallest weight `w` (`0 <= w <= 1`)
    /// at which `start * (1 - w) + end * w` is within this shape.
    pub fn between(&self, start: Point, end: Point) -> Option<f64> {
        match self {
            Self::Cube => {
                let dir = end - start;

                let mut min_w = None;
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
                                min_w = Some(match min_w {
                                    Some(prev) if prev < w => prev,
                                    _ => w,
                                });
                            }
                        }
                    }
                }

                min_w
            }
            Self::Sphere => {
                if self.contains(start) {
                    return Some(0.);
                }
                if self.contains(end) {
                    return Some(1.); // FIXME: This is not the closest point!
                }

                let dir = end - start;

                // Neither endpoint is within the sphere.
                // If the sphere contains part of the segment, the closest point of the line
                // containing the segment from the sphere center must also be within the segment.
                let w = (Point::origin() - start).dot(&dir) / dir.norm_squared();
                let closest = start + dir * w;
                self.contains(closest).then(|| w)
            }
            Self::Cylinder => {
                // Tests whether the line intersects with the unit circle on the XY plane
                // Formula: https://www.wolframalpha.com/input/?i=%28x_1%2Bx_3+w%29%5E2+%2B+%28y_1%2B+y_3+w+%29%5E2%3D1
                // where `x1` = `start.x`, `x3` = `delta.x`
                // where `y1` = `start.y`, `y3` = `delta.y`
                let delta = end - start;
                let dxy2 = delta.x.powi(2) + delta.y.powi(2);
                let discrim = (start.x * delta.x + start.y * delta.y).powi(2)
                    - (start.x.powi(2) + start.y.powi(2) - 1.) * dxy2;

                if discrim > 0. {
                    let dr = discrim.sqrt();
                    let base = start.x * delta.x + start.y * delta.y;

                    // the less weight intersecting with the unit circle
                    let w1 = (-base - dr) / dxy2;
                    // the greater weight intersecting with the unit circle
                    let w2 = (-base + dr) / dxy2;

                    // the weight intersecting with the lower plane
                    let zw0 = -start.z / delta.z;
                    // the weight intersecting with the higher plane
                    let zw1 = (1. - start.z) / delta.z;

                    fn intersect_ranges(
                        a: RangeInclusive<f64>,
                        b: RangeInclusive<f64>,
                    ) -> Option<RangeInclusive<f64>> {
                        if a.end() < b.start() || b.end() < a.start() {
                            return None;
                        }
                        (a.end() >= b.start() && b.end() >= a.start()).then(|| {
                            f64::max(*a.start(), *b.start())..=f64::min(*a.end(), *b.end())
                        })
                    }

                    let range = intersect_ranges(
                        intersect_ranges(0. ..=1., w1..=w2)?,
                        f64::min(zw0, zw1)..=f64::max(zw0, zw1),
                    )?;
                    Some(*range.start())
                } else {
                    None
                }
            }
        }
    }

    /// Computes the axis-aligned bounding box under the given transformation matrix
    ///
    /// The transformation matrix should transform the unit shape to the real coordinates.
    #[allow(clippy::indexing_slicing)]
    pub fn bb_under(&self, transform: Matrix) -> (Point, Point) {
        use nalgebra::dimension as dim;

        match self {
            Self::Cube => {
                type Storage = nalgebra::storage::Owned<f64, dim::U4, dim::U8>;
                type Points = nalgebra::Matrix<f64, dim::U4, dim::U8, Storage>;

                fn p01() -> impl Iterator<Item = f64> { [0., 1.].iter().copied() }
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
                (Point::new(min[0], min[1], min[2]), Point::new(max[0], max[1], max[2]))
            }
            Self::Sphere => {
                // Extremize f(x,y,z) := ax+by+cz+d under g(x,y,z) := x^2+y^2+z^2-1 = 0
                // By Lagrange multipliers theorem,
                // solving d/d[xyz] f(x,y,z) = lambda * d/d[xyz] g(x,y,z)
                // gives the following equations for a,b,c not all zero:
                // x = \pm a / sqrt(a^2+b^2+c^2)
                // y = \pm b / sqrt(a^2+b^2+c^2)
                // z = \pm c / sqrt(a^2+b^2+c^2)

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

                let min =
                    Point::from(Vector::from_iterator(extrema.iter().map(|&(i, j)| i.min(j))));
                let max =
                    Point::from(Vector::from_iterator(extrema.iter().map(|&(i, j)| i.max(j))));

                (min, max)
            }
            Self::Cylinder => {
                // Extremize f(x,y) := ax+by+cZ+d under g(x,y) := x^2+y^2-1 = 0,
                // where Z is 0 or 1.
                // solving d/d[xy] f(x,y) = lambda * d/d[xy] g(x,y)
                // gives the following equations for a,b not all zero:
                // x = \pm a / sqrt(a^2+b^2)
                // y = \pm b / sqrt(a^2+b^2)

                let extrema: SmallVec<[SmallVec<[f64; 4]>; 3]> = (0_usize..3)
                    .map(|i| {
                        let row = transform.row(i);

                        let norm = row.fixed_slice::<1, 2>(0, 0).norm();
                        if norm.abs() < 1e-10 {
                            return smallvec![row[3], row[2] + row[3], row[3], row[2] + row[3]];
                        }

                        let points: SmallVec<[f64; 4]> = [-1_f64, 1.]
                            .iter()
                            .flat_map(|&sgn| [(sgn, 0_f64), (sgn, 1_f64)])
                            .map(|(sgn, z)| {
                                let unit = nalgebra::Vector4::new(
                                    sgn * row[0] / norm,
                                    sgn * row[1] / norm,
                                    z,
                                    1.,
                                );
                                (row * unit)[0]
                            })
                            .collect();
                        points
                    })
                    .collect();

                let min = Point::from(Vector::from_iterator(
                    extrema.iter().map(|array| array.iter().copied().fold(array[0], f64::min)),
                ));
                let max = Point::from(Vector::from_iterator(
                    extrema.iter().map(|array| array.iter().copied().fold(array[0], f64::max)),
                ));

                (min, max)
            }
        }
    }
}

/// Returns the coordinates of a sprite in the spritesheet of a given size.
///
/// This encoding scheme ensures that the return value of `order`
/// is homogeneous over decreasing values of `side`.
fn sprite_location(order: u32, side: u32) -> (u32, u32) {
    if order == 0 {
        return (0, 0);
    }
    let subarea = side * side / 4;
    let (x, y) = sprite_location(order % subarea, side / 2);
    match order / subarea {
        0 => (x, y),
        1 => (x, y + side / 2),
        2 => (x + side / 2, y),
        3 => (x + side / 2, y + side / 2),
        _ => unreachable!("order > side * side"),
    }
}

/// Returns the side length of a spritesheet given the number of items.
fn spritesheet_side(max_order: u32) -> u32 {
    // Find the smallest power of 4 greater than `order`.
    let power_of_four = u32::BITS - max_order.leading_zeros();
    let power_of_two = (power_of_four + 1) / 2;
    1 << power_of_two
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{PI, SQRT_2};
    use std::ops::Range;

    use super::Unit;
    use crate::space::{Matrix, Point, Vector};

    fn assert_pt(pt: Point, [x, y, z]: [f64; 3]) {
        let a = &pt.coords;
        let b = &Vector::new(x, y, z);
        let delta = (a - b).norm();

        if !pt.coords.map(f64::is_finite).fold(true, |a, b| a && b) {
            panic!("Point is not finite: {}", pt);
        }

        if delta > 1e-10 {
            panic!("{} != {}", a, b);
        }
    }

    fn assert_bb(unit: Unit, trans: Matrix, range: Range<[f64; 3]>) {
        let bb = unit.bb_under(trans);
        assert_pt(bb.0, range.start);
        assert_pt(bb.1, range.end);
    }

    #[test]
    pub fn sphere_bb() {
        assert_bb(Unit::Sphere, Matrix::identity(), [-1., -1., -1.]..[1., 1., 1.]);
        assert_bb(
            Unit::Sphere,
            Matrix::new_translation(&Vector::new(0.5, 0.5, 0.5)),
            [-0.5, -0.5, -0.5]..[1.5, 1.5, 1.5],
        );
        assert_bb(
            Unit::Sphere,
            Matrix::new_nonuniform_scaling(&Vector::new(0.5, 2., 5.)),
            [-0.5, -2., -5.]..[0.5, 2., 5.],
        );

        assert_bb(
            Unit::Sphere,
            nalgebra::Rotation3::from_axis_angle(&Vector::x_axis(), PI / 2.)
                .to_homogeneous()
                .prepend_translation(&Vector::new(1., 1., 1.)),
            [0., -2., 0.]..[2., 0., 2.],
        );
    }

    #[test]
    pub fn cylinder_bb() {
        assert_bb(Unit::Cylinder, Matrix::identity(), [-1., -1., 0.]..[1., 1., 1.]);
        assert_bb(
            Unit::Cylinder,
            Matrix::new_translation(&Vector::new(0.5, 0.5, -0.5))
                .append_nonuniform_scaling(&Vector::new(2., 3., 4.)),
            [-1., -1.5, -2.]..[3., 4.5, 2.],
        );
        assert_bb(
            Unit::Cylinder,
            nalgebra::Rotation3::from_axis_angle(&Vector::x_axis(), PI / 4.)
                .matrix()
                .to_homogeneous()
                .prepend_translation(&Vector::new(0., 0., -0.5)),
            [-1., -0.75 * SQRT_2, -0.75 * SQRT_2]..[1., 0.75 * SQRT_2, 0.75 * SQRT_2],
        );
    }

    #[test]
    pub fn cylinder_between() {
        macro_rules! assert_between {
            (($x0:expr, $y0:expr, $z0:expr)..($x1:expr, $y1:expr, $z1:expr) => None) => {
                let v0 = Point::new($x0, $y0, $z0);
                let v1 = Point::new($x1, $y1, $z1);
                let option = Unit::Cylinder.between(v0, v1);
                if let Some(w) = option {
                    panic!("{}..{} should not intersect cylinder, got Some({})", v0, v1, w);
                }
            };
            (($x0:expr, $y0:expr, $z0:expr)..($x1:expr, $y1:expr, $z1:expr) => Some($w:expr, $eps:expr)) => {
                let v0 = Point::new($x0, $y0, $z0);
                let v1 = Point::new($x1, $y1, $z1);
                let option = Unit::Cylinder.between(v0, v1);
                if let Some(w) = option {
                    if ($w - w).abs() > $eps {
                        panic!(
                            "{}..{} should intersect cylinder at {} (\u{00b1} {}, got Some({})",
                            v0, v1, $w, $eps, w
                        );
                    }
                } else {
                    panic!("{}..{} should intersect cylinder, got None", v0, v1);
                }
            };
        }

        assert_between!((-1., -1., 0.5)..(1., 1., 0.5) => Some((2f64.sqrt() - 1.) / (2f64.sqrt() * 2.), 1e-6));
        assert_between!((-1., -1., -0.5)..(1., 1., -0.5) => None);
        assert_between!((1., -1., 0.5)..(-1., 1., 0.5) => Some((2f64.sqrt() - 1.) / (2f64.sqrt() * 2.), 1e-6));
        assert_between!((-1., 1., 0.5)..(1., -1., 0.5) => Some((2f64.sqrt() - 1.) / (2f64.sqrt() * 2.), 1e-6));
        assert_between!((-2., 0., 0.5)..(0., 0., 0.5) => Some(0.5, 1e-10));
        assert_between!((0., 0., 0.5)..(2., 0., 0.5) => Some(0., 1e-10));
        assert_between!((0., 0., 1.5)..(1., 0., 0.5) => Some(0.5, 1e-10));
        assert_between!((1., 0., 1.5)..(0., 0., 0.5) => Some(0.5, 1e-10));
        assert_between!((0., 0., 0.5)..(1., 0., 1.5) => Some(0., 1e-10));
        assert_between!((1., 0., 0.5)..(0., 0., 1.5) => Some(0., 1e-10));

        assert_between!((0., 0., 11.)..(37., -12., -753.) => Some((1. - 11.) / (-753. - 11.), 1e-10));
    }
}
