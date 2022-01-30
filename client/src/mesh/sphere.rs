use std::f32::consts::PI;
use std::mem;

use xias::Xias;

const NUM_STEPS: usize = 32;

pub fn compute() -> three_d::CPUMesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    let mut indices = Vec::new();
    let mut index_counter: u16 = 0;

    macro_rules! push_vertex {
        ($pos:expr, $theta:expr, $phi:expr) => {{
            positions.extend_from_slice(&[$pos[0], $pos[1], $pos[2]]);
            normals.extend_from_slice(&[$pos[0], $pos[1], $pos[2]]);
            uvs.extend_from_slice(&[$phi / PI / 2., $theta / PI]);

            let index = index_counter;
            index_counter += 1;
            index
        }};
    }

    let top = push_vertex!([0., 0., 1.], 0., 0.);
    let bottom = push_vertex!([0., 0., -1.], PI, 0.);

    let mut previous = vec![top; NUM_STEPS];
    let mut current = vec![0; NUM_STEPS];

    for i in 1..NUM_STEPS {
        let theta = i.small_float::<f32>() * PI / NUM_STEPS.small_float::<f32>();
        let (theta_sin, theta_cos) = theta.sin_cos();

        #[allow(clippy::needless_range_loop)]
        for j in 0..NUM_STEPS {
            let phi = j.small_float::<f32>() * PI * 2. / NUM_STEPS.small_float::<f32>();
            let (phi_sin, phi_cos) = phi.sin_cos();

            let vert =
                push_vertex!([theta_sin * phi_cos, theta_sin * phi_sin, theta_cos], theta, phi);
            current[j] = vert;
        }

        for j in 0..NUM_STEPS {
            let jj = if j + 1 == NUM_STEPS { 0 } else { j + 1 };

            indices.extend_from_slice(&[previous[j], previous[jj], current[j]]);
            indices.extend_from_slice(&[current[j], previous[jj], current[jj]]);

            if i + 1 == NUM_STEPS {
                indices.extend_from_slice(&[current[j], current[jj], bottom]);
            }
        }

        mem::swap(&mut previous, &mut current);
    }

    three_d::CPUMesh {
        name: "traffloat.sphere".to_string(),
        positions,
        normals: Some(normals),
        uvs: Some(uvs),
        indices: Some(three_d::Indices::U16(indices)),
        ..Default::default()
    }
}
