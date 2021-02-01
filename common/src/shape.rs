//! Shape and appearance of an object

use crate::types::{Config, ConfigStore, Id, Matrix, Point, Position};
use crate::SetupEcs;

/// Describes the shape and appearance of an object
pub struct Shape {
    /// Unit shape variant
    pub unit: Unit,
    /// The transformation matrix from the unit square to this shape centered at the
    /// origin
    pub matrix: Matrix,
    /// The texture for rendering the shape
    pub texture: Id<Texture>,
}

impl Shape {
    /// The transformation matrix from the unit square to this shape centered at pos
    pub fn transform(&self, pos: Position) -> Matrix {
        self.matrix.append_translation(&pos.vector())
    }
}

/// A unit shape variant
pub enum Unit {
    /// A unit square `[0, 1]^2`
    Square,
    /// A unit circle `x^2 + y^2 <= 1`
    Circle,
}

impl Unit {
    /// Checks whether the given point is within this unit shape
    pub fn contains(&self, pos: Point) -> bool {
        match self {
            Self::Square => (0. ..=1.).contains(&pos.x) && (0. ..=1.).contains(&pos.y),
            Self::Circle => pos.x.powi(2) + pos.y.powi(2) <= 1.,
        }
    }

    /// Computes the axis-aligned bounding box under the given transformation matrix
    ///
    /// The transformation matrix should transform the unit shape to the real coordinates.
    pub fn bb_under(&self, transform: Matrix) -> (Point, Point) {
        fn fmax(a: f64, b: f64) -> f64 {
            if a > b {
                a
            } else {
                b
            }
        }
        fn fmin(a: f64, b: f64) -> f64 {
            if a < b {
                a
            } else {
                b
            }
        }
        match self {
            Self::Square => {
                let (x0, x1, y0, y1) = [0_f64, 1.]
                    .iter()
                    .flat_map(|&x| [0_f64, 1.].iter().map(move |&y| Point::new(x, y)))
                    .map(|point| transform.transform_point(&point))
                    .fold(None, |opt, pt| match opt {
                        Some((x0, x1, y0, y1)) => Some((
                            fmin(x0, pt.x),
                            fmax(x1, pt.x),
                            fmin(y0, pt.y),
                            fmax(y1, pt.y),
                        )),
                        None => Some((pt.x, pt.x, pt.y, pt.y)),
                    })
                    .expect("nonempty iterator");
                (Point::new(x0, y0), Point::new(x1, y1))
            }
            Self::Circle => {
                fn circle_extrema(a: f64, b: f64, c: f64) -> (f64, f64) {
                    let candidates = &[
                        c,
                        c + a.abs(),
                        c + (a * a + b * b).sqrt(),
                        c - a.abs(),
                        c - (a * a + b * b).sqrt(),
                    ];
                    let iter = candidates.iter().copied().filter(|f| f.is_finite());
                    (
                        iter.clone()
                            .fold_first(fmin)
                            .expect("candidates is nonempty"),
                        iter.fold_first(fmax).expect("candidates is nonempty"),
                    )
                }

                let (x0, x1) = circle_extrema(
                    *transform.index((0, 0)),
                    *transform.index((0, 1)),
                    *transform.index((0, 2)),
                );
                let (y0, y1) = circle_extrema(
                    *transform.index((1, 0)),
                    *transform.index((1, 1)),
                    *transform.index((1, 2)),
                );

                (Point::new(x0, y0), Point::new(x1, y1))
            }
        }
    }
}

/// The texture of a rendered object
#[derive(Debug)]
pub struct Texture {
    /// A URL compatible with `<img src>`
    pub url: String,
}

impl Config for Texture {}

/// Initializes systems
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.resource(ConfigStore::<Texture>::default())
}
