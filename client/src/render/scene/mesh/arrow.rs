//! This module generates geometry data for a 3D arrow.
#![allow(clippy::indexing_slicing)] // The whole module is only executed on startup, no worries

use lazy_static::lazy_static;

const PRISM_SCALE: f32 = 0.01;
const TIP_SCALE: f32 = 0.04;
const PRISM_HEIGHT: f32 = 0.8;
const TIP_HEIGHT: f32 = 0.2;

lazy_static! {
    /// Geometry data for a 3D arrow.
    pub static ref ARROW: Vec<f32> = {
        let top_corner: [f32; 2] = [0., 2.];
        let left_corner: [f32; 2] = [-(3f32.sqrt()), -1.];
        let right_corner: [f32; 2] = [3f32.sqrt(), -1.];
        let corners = [top_corner, left_corner, right_corner];

        let mut ret = Vec::new();
        for edge in 0..3 {
            let v1 = corners[edge];
            let v2 = corners[(edge + 1) % 3];

            ret.extend(&[0., v1[0] * PRISM_SCALE, v1[1] * PRISM_SCALE]);
            ret.extend(&[PRISM_HEIGHT, v1[0] * PRISM_SCALE, v1[1] * PRISM_SCALE]);
            ret.extend(&[0., v2[0] * PRISM_SCALE, v2[1] * PRISM_SCALE]);

            ret.extend(&[PRISM_HEIGHT, v2[0] * PRISM_SCALE, v2[1] * PRISM_SCALE]);
            ret.extend(&[PRISM_HEIGHT, v1[0] * PRISM_SCALE, v1[1] * PRISM_SCALE]);
            ret.extend(&[0., v2[0] * PRISM_SCALE, v2[1] * PRISM_SCALE]);

            ret.extend(&[PRISM_HEIGHT + TIP_HEIGHT, 0., 0.]);
            ret.extend(&[PRISM_HEIGHT, -v1[0] * TIP_SCALE, -v1[1] * TIP_SCALE]);
            ret.extend(&[PRISM_HEIGHT, -v2[0] * TIP_SCALE, -v2[1] * TIP_SCALE]);
        }

        ret
    };
}
