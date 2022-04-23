use std::f32::consts::PI;

use nalgebra::{Vector2, Vector3};
use traffloat_types::geometry;
use xias::Xias;

const NUM_VERT: u32 = 32;
const SPRITESHEET_DIM: f32 = 2.; // 2 or 3 sprites, fits on a 2^2 spritesheet.

pub fn compute(fused: bool) -> three_d::CPUMesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    let mut indices = Vec::new();
    let mut index_counter: u16 = 0;

    macro_rules! push_vertex {
        (position: $position:expr, normal: $normal:expr, uv: $uv:expr,) => {{
            positions.extend_from_slice($position);
            normals.extend_from_slice($normal);
            uvs.extend_from_slice($uv);

            let index = index_counter;
            index_counter += 1;
            index
        }};
    }

    if fused {
        for (name, z) in [("bottom", 0.), ("top", 1.)] {
            let uv0 = geometry::Unit::Cylinder
                .search_sprite_coord_by_name(name)
                .expect("Unit::sprite_name should return top, bottom and curved");
            let uv0 = Vector2::new(uv0.0.small_float(), uv0.1.small_float());
            let uv_fn =
                |pos: Vector2<f32>| (uv0 + Vector2::new(0.5, 0.5) + pos * 0.5) / SPRITESHEET_DIM;

            let normal = Vector3::new(0., 0., z * 2. - 1.);

            let center = push_vertex! {
                position: &[0., 0., z],
                normal: normal.as_slice(),
                uv: uv_fn(Vector2::new(0., 0.)).as_slice(),
            };

            let mut last_vert_index = push_vertex! {
                position: &[1., 0., z],
                normal: normal.as_slice(),
                uv: uv_fn(Vector2::new(1., 0.)).as_slice(), // cos(0), sin(0)
            };

            for vert in 1..=NUM_VERT {
                let mut angle = PI * 2. / NUM_VERT.small_float::<f32>() * vert.small_float::<f32>();
                angle *= z * 2. - 1.; // negate the angle if z == 0.

                let (y, x) = angle.sin_cos();
                let pos2 = Vector2::new(x, y);

                let vert_index = push_vertex! {
                    position: &[x, y, z],
                    normal: normal.as_slice(),
                    uv: uv_fn(pos2).as_slice(),
                };

                indices.extend_from_slice(&[center, last_vert_index, vert_index]);
                last_vert_index = vert_index;
            }
        }
    }

    // curved vertices need new vertices because of different normal
    let curved_uv0 = geometry::Unit::Cylinder
        .search_sprite_coord_by_name("curved")
        .expect("Cylinder has curved sprite");
    let curved_uv0 = Vector2::new(curved_uv0.0.small_float(), curved_uv0.1.small_float());

    macro_rules! push_side_vertex {
        ($vert:expr, $z: expr) => {{
            let ratio = $vert.small_float::<f32>() / NUM_VERT.small_float::<f32>();
            let angle = PI * 2. * ratio;

            let (y, x) = angle.sin_cos();

            push_vertex! {
                position: &[x, y, $z],
                normal: &[x, y, 0.],
                uv: ((curved_uv0 + Vector2::new(ratio, $z)) / SPRITESHEET_DIM).as_slice(),
            }
        }};
    }

    let mut top0 = push_side_vertex!(0, 1.);
    let mut bottom0 = push_side_vertex!(0, 0.);

    for vert in 1..=NUM_VERT {
        let top1 = push_side_vertex!(vert, 1.);
        let bottom1 = push_side_vertex!(vert, 0.);

        indices.extend_from_slice(&[top0, bottom0, top1]);
        indices.extend_from_slice(&[bottom0, bottom1, top1]);

        top0 = top1;
        bottom0 = bottom1;
    }

    three_d::CPUMesh {
        name: if fused { "traffloat.fused-cylinder" } else { "traffloat.cylinder" }.to_string(),
        positions,
        normals: Some(normals),
        uvs: Some(uvs),
        indices: Some(three_d::Indices::U16(indices)),
        ..Default::default()
    }
}
