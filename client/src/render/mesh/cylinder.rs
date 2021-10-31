//! A cylinder model.

use std::f32::consts::PI;

use nalgebra::{Vector2, Vector3};
use safety::Safety;
use traffloat::appearance;
use typed_builder::TypedBuilder;
use web_sys::WebGlRenderingContext;

/// Options for cylinder model generation.
#[derive(TypedBuilder)]
pub struct Options {
    /// Number of vertices for each circle.
    num_vert: u32,
    /// Whether the top and bottom sides should be included.
    fused:    bool,
}

/// Creates a cylinder model.
pub fn prepare(gl: &WebGlRenderingContext, options: Options) -> impl super::Mesh {
    let mut builder = super::Builder::default();

    if options.fused {
        for (name, z) in [("bottom", 0.), ("top", 1.)] {
            let tex_pos = appearance::Unit::Cylinder
                .search_sprite_coord_by_name(name)
                .expect("Unit::sprite_name should return top, bottom and curved");
            let tex_pos = Vector2::new(tex_pos.0.small_float(), tex_pos.1.small_float());

            let normal = Vector3::new(0., 0., z * 2. - 1.);
            let center =
                builder.push(Vector3::new(0., 0., z), normal, tex_pos + Vector2::new(0.5, 0.5));

            let mut last_vert_index =
                builder.push(Vector3::new(1., 0., z), normal, tex_pos + Vector2::new(1., 0.));
            for vert in 0..options.num_vert {
                let mut angle = PI * 2. / options.num_vert.small_float() * vert.small_float();
                angle *= z * 2. - 1.; // negate the angle if z == 0.

                let pos2 = Vector2::new(angle.cos(), angle.sin());

                let vert_index = builder.push(pos2.fixed_resize(z), normal, tex_pos + pos2);
                builder.push_triangle([center, last_vert_index, vert_index]);
                last_vert_index = vert_index;
            }
        }
    }

    // curved vertices need new vertices because of different normal
    let curved_tex_pos = appearance::Unit::Cylinder
        .search_sprite_coord_by_name("curved")
        .expect("Cylinder has curved sprite");
    let curved_tex_pos =
        Vector2::new(curved_tex_pos.0.small_float(), curved_tex_pos.1.small_float());

    let push_side_vertex = |builder: &mut super::Builder, vert: u32, z: f32| {
        let ratio = vert.small_float() / options.num_vert.small_float();
        let angle = PI * 2. * ratio;

        builder.push(
            Vector3::new(angle.cos(), angle.sin(), z),
            Vector3::new(angle.cos(), angle.sin(), 0.),
            curved_tex_pos + Vector2::new(ratio, z),
        )
    };

    let mut top0 = push_side_vertex(&mut builder, 0, 1.);
    let mut bottom0 = push_side_vertex(&mut builder, 0, 0.);

    for vert in 1..=options.num_vert {
        let top1 = push_side_vertex(&mut builder, vert, 1.);
        let bottom1 = push_side_vertex(&mut builder, vert, 0.);

        builder.push_triangle([top0, top1, bottom0]);
        builder.push_triangle([bottom0, top1, bottom1]);

        top0 = top1;
        bottom0 = bottom1;
    }

    builder.compile_indexed(gl)
}
