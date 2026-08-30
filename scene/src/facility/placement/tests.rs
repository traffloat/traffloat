use std::f32::consts::SQRT_2;

use bevy::math::Vec3;
use bevy::transform::components::Transform;

#[test]
fn test_compute_placement() {
    let placement: Vec<_> = super::compute(1).collect();
    assert_eq!(placement.len(), 1);
    assert_eq!(placement[0], Transform::IDENTITY.with_scale(Vec3::new(SQRT_2, SQRT_2, 1.0)));
}
