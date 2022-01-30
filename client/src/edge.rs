use std::error::Error;

use nalgebra::Rotation3;
use three_d::GeometryMut;
use traffloat_def::edge;
use traffloat_types::space::{Matrix, Position, Vector};
use xias::Xias;

use crate::mat;

pub struct View {
    pub id:     edge::AlphaBeta,
    pub radius: f64,
    pub color:  [f32; 4],
}

pub struct Prepared {
    pub view:  View,
    pub model: three_d::Model<three_d::PhysicalMaterial>,
}

impl Prepared {
    pub fn new(
        view: View,
        cylinder: &three_d::CPUMesh,
        gl: &three_d::Context,
        endpoints: [Position; 2],
    ) -> Result<Self, Box<dyn Error>> {
        let material = three_d::PhysicalMaterial::new(
            gl,
            &three_d::CPUMaterial { metallic: 0.8, roughness: 0.2, ..Default::default() },
        )?;

        let mut this =
            Self { view, model: three_d::Model::new_with_material(gl, cylinder, material)? };
        this.set_endpoints(endpoints[0], endpoints[1]);

        this.set_color(this.view.color);

        Ok(this)
    }

    pub fn set_endpoints(&mut self, alpha: Position, beta: Position) {
        let diff = beta.value() - alpha.value();
        let mut tf = match Rotation3::rotation_between(&Vector::new(0., 0., 1.), &diff) {
            Some(rot) => rot.to_homogeneous(),
            _ => Matrix::identity(),
        };
        tf.prepend_nonuniform_scaling_mut(&Vector::new(
            self.view.radius,
            self.view.radius,
            diff.norm(),
        ));
        tf.append_translation_mut(&alpha.vector());
        self.model.set_transformation(mat(tf));
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.model.material.albedo = three_d::Color::new(
            (color[0] * 255.).trunc_int(),
            (color[1] * 255.).trunc_int(),
            (color[2] * 255.).trunc_int(),
            (color[3] * 255.).trunc_int(),
        );
    }

    pub fn object(&self) -> &dyn three_d::Object { &self.model as &dyn three_d::Object }
}
