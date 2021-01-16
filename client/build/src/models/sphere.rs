use std::collections::BTreeMap;
use std::f32::consts::PI;

use super::*;
use traffloat_client_model::FaceIndex;

pub(super) fn sphere(depth: u32) -> (Vec<Vertex>, Vec<Face>) {
    let mut vertices: Vec<UnitVertex> = Vec::with_capacity(4);
    {
        vertices.push(UnitVertex {
            theta: 0.,
            phi: PI / 2.,
        });
        let base_phi = PI / 2. - 2. * (2.0_f32).sqrt().atan();
        for i in 0..3 {
            vertices.push(UnitVertex {
                theta: PI / 3. * (i as f32),
                phi: base_phi,
            });
        }
    }

    let mut faces = vec![
        Face([0, 1, 2]),
        Face([0, 1, 3]),
        Face([0, 2, 3]),
        Face([1, 2, 3]),
    ];

    let mut midpt_index = BTreeMap::new();

    fn midpt(
        vertices: &mut Vec<UnitVertex>,
        midpt_index: &mut BTreeMap<(usize, usize), usize>,
        a: FaceIndex,
        b: FaceIndex,
    ) -> usize {
        *midpt_index
            .entry((a as usize, b as usize))
            .or_insert_with(|| {
                let a: UnitVertex = vertices[a as usize];
                let b: UnitVertex = vertices[b as usize];
                vertices.push(a.midpt(b));
                vertices.len() - 1
            })
    }

    for _ in 0..depth {
        for i in 0..faces.len() {
            let Face([a, b, c]) = faces[i];
            let ab = midpt(&mut vertices, &mut midpt_index, a, b) as FaceIndex;
            let bc = midpt(&mut vertices, &mut midpt_index, b, c) as FaceIndex;
            let ca = midpt(&mut vertices, &mut midpt_index, c, a) as FaceIndex;
            faces[i] = Face([ab, bc, ca]);
            faces.push(Face([a, ab, ca]));
            faces.push(Face([b, bc, ab]));
            faces.push(Face([c, ca, bc]));
        }
    }

    let vertices = vertices.into_iter().map(Vertex::from).collect();
    (vertices, faces)
}

#[derive(Debug, Clone, Copy)]
struct UnitVertex {
    theta: f32,
    phi: f32,
}

impl UnitVertex {
    fn midpt(self, other: Self) -> Self {
        Self {
            theta: angle_midpt(self.theta, other.theta),
            phi: angle_midpt(self.phi, other.phi),
        }
    }
}

const CIRCLE: f32 = PI * 2.;

fn angle_std(mut a: f32) -> f32 {
    a %= CIRCLE;
    if a < 0. {
        a += CIRCLE;
    }
    a
}

fn angle_midpt(mut a: f32, mut b: f32) -> f32 {
    a = angle_std(a);
    b = angle_std(b);
    let mut midpt = (a + b) / 2.;
    if angle_std((midpt - a).abs()) > PI / 2. {
        midpt += PI;
    }
    angle_std(midpt)
}

#[cfg(test)]
#[test]
fn midpt_tests() {
    macro_rules! check {
        ($a:expr, $b:expr => $expect:expr) => {
            let midpt = angle_midpt($a, $b);
            let delta: f32 = midpt - $expect;
            if delta.abs() > 1e-9 {
                panic!(
                    "Assertion failed: midpt({}, {}) != {}, got {}",
                    $a / PI,
                    $b / PI,
                    $expect / PI,
                    midpt / PI
                );
            }
        };
    }

    check!(0., CIRCLE => 0.);
    check!(0., PI * 1.5 => PI * 1.75);
    check!(CIRCLE, PI * 1.5 => PI * 1.75);
    check!(PI * 0.75, PI * 1.25 => PI);

    check!(CIRCLE, 0. => 0.);
    check!(PI * 1.5, 0. => PI * 1.75);
    check!(PI * 1.5, CIRCLE => PI * 1.75);
    check!(PI * 1.25, PI * 0.75 => PI);
}

impl From<UnitVertex> for Vertex {
    fn from(v: UnitVertex) -> Self {
        Self([
            v.theta.cos() * v.phi.cos(),
            v.theta.cos() * v.phi.sin(),
            v.theta.sin(),
        ])
    }
}
