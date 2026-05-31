#![expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "n is a small positive integer"
)]

use std::f32::consts::{FRAC_1_SQRT_2, SQRT_2};

use bevy::math::Vec3;
use bevy::transform::components::Transform;
use either::Either;

#[cfg(test)]
mod tests;

/// Compute the placement of n squares within a circle of radius 1 centered at origin.
///
/// `n` must be a positive number.
pub(super) fn compute(n: usize) -> impl ExactSizeIterator<Item = Transform> {
    // Number of squares on each side.
    // (side-1)^2 < n <= side^2 must hold due to the properties of `ceil`.
    let side = (n as f32).sqrt().ceil();
    let side_int = side as usize;

    // In both cases, `0 < num_full_rows <= num_rows` holds.
    let num_rows;
    let num_full_rows;
    if n <= side_int * (side_int - 1) {
        // We will place `side-1` rows.
        // The first `n - (side-1)^2` rows will have `side` cols, the rest have `isde-1 cols.
        num_rows = side_int - 1;
        num_full_rows = n - (side_int - 1).pow(2);
        // Since `side*(side-1) = (side-1)^2 + (side-1) >= n`,
        // `0 < num_full_rows  <= side-1 = num_rows`.
    } else {
        // We will place `side` rows.
        // The last `side^2 - n` rows will have `side-1` cols, the rest have `side` cols.
        num_rows = side_int;
        let num_empty_rows = side_int * side_int - n;
        num_full_rows = num_rows - num_empty_rows;
        // Since `n > side^2 - side`, `side^2 - n < side`,
        // so `0 < side - (side^2 - n) = num_full_rows <= side = num_rows`.
    }

    let mut y = 0;
    let mut y_cols = side_int; // since num_full_rows > 0, first row must have `side` cols.
    let mut x = 0;
    (0..n)
        .map(move |_| {
            let out = (x, y, y_cols);
            x += 1;
            if x == y_cols {
                x = 0;
                y += 1;
                y_cols = if y < num_full_rows { side_int } else { side_int - 1 };
            }
            out
        })
        .map(move |(x, y, cols)| make_transform(x, y, cols, num_rows, side))
}

fn make_transform(x: usize, y: usize, cols: usize, rows: usize, side: f32) -> Transform {
    let x = midpt_1d(cols, x) * FRAC_1_SQRT_2;
    let y = midpt_1d(rows, y) * FRAC_1_SQRT_2;
    let scale = SQRT_2 / side;
    Transform::from_translation(Vec3::new(x, y, 0.0)).with_scale(Vec3::new(scale, scale, 1.0))
}

fn midpt_1d(total: usize, index: usize) -> f32 {
    (2.0 * (index as f32) + 1.0) / (total as f32) - 1.0
}
