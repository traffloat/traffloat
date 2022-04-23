use std::error::Error;

use nalgebra::Rotation3;
use three_d::{GeometryMut, ThreeDResult};
use traffloat_def::edge;
use traffloat_types::space::{Matrix, Position, Vector};

use crate::mat;

pub struct View {
    pub id:        edge::AlphaBeta,
    pub radius:    f64,
    pub metallic:  f32,
    pub roughness: f32,
    pub color:     [f32; 4],
    pub ducts:     Vec<Duct>,
}

#[derive(Default)]
pub struct Duct {
    pub position:  (f64, f64),
    pub radius:    f64,
    pub metallic:  f32,
    pub roughness: f32,
    pub color:     [f32; 4],
}

pub struct Prepared {
    pub view:  View,
    models:    Vec<three_d::Model<three_d::PhysicalMaterial>>,
    endpoints: [Position; 2],
    picked:    bool,
}

fn reconstruct_models(
    gl: &three_d::Context,
    cylinder: &three_d::CPUMesh,
    view: &View,
    tf: Matrix,
    picked: bool,
) -> ThreeDResult<Vec<three_d::Model<three_d::PhysicalMaterial>>> {
    let mut models = Vec::with_capacity(1 + view.ducts.len());

    models.push({
        let mut color = view.color;
        if !picked {
            for ch in &mut color[0..3] {
                *ch *= 0.8;
            }
        }
        let color = three_d::Color::from_rgba_slice(&color);

        let material = three_d::PhysicalMaterial::new(
            gl,
            &three_d::CPUMaterial {
                metallic: view.metallic,
                roughness: view.roughness,
                albedo: color,
                ..Default::default()
            },
        )?;
        let mut model = three_d::Model::new_with_material(gl, cylinder, material)?;
        model.set_transformation(mat(tf.prepend_nonuniform_scaling(&Vector::new(
            view.radius,
            view.radius,
            1.,
        ))));
        model
    });

    for duct in &view.ducts {
        models.push({
            let material = three_d::PhysicalMaterial::new(
                gl,
                &three_d::CPUMaterial {
                    metallic: duct.metallic,
                    roughness: duct.roughness,
                    albedo: three_d::Color::from_rgba_slice(&duct.color),
                    ..Default::default()
                },
            )?;
            let mut model = three_d::Model::new_with_material(gl, cylinder, material)?;
            model.set_transformation(mat(tf
                .prepend_translation(&Vector::new(duct.position.0, duct.position.1, 0.))
                .prepend_nonuniform_scaling(&Vector::new(duct.radius, duct.radius, 1.))));
            model
        });
    }

    Ok(models)
}

fn ab_to_tf(alpha: Position, beta: Position) -> Matrix {
    let diff = beta.value() - alpha.value();

    let mut tf = match Rotation3::rotation_between(&Vector::new(0., 0., 1.), &diff) {
        Some(rot) => rot.to_homogeneous(),
        _ => Matrix::identity(),
    };

    tf.prepend_nonuniform_scaling_mut(&Vector::new(1., 1., diff.norm()));

    tf.append_translation_mut(&alpha.vector());

    tf
}

impl Prepared {
    pub fn new(
        view: View,
        cylinder: &three_d::CPUMesh,
        gl: &three_d::Context,
        endpoints: [Position; 2],
    ) -> Result<Self, Box<dyn Error>> {
        let tf = ab_to_tf(endpoints[0], endpoints[1]);
        let models = reconstruct_models(gl, cylinder, &view, tf, false)?;

        let this = Self { view, models, endpoints, picked: false };

        Ok(this)
    }

    fn reconstruct_models(
        &mut self,
        gl: &three_d::Context,
        cylinder: &three_d::CPUMesh,
    ) -> ThreeDResult<()> {
        let tf = ab_to_tf(self.endpoints[0], self.endpoints[1]);
        self.models = reconstruct_models(gl, cylinder, &self.view, tf, self.picked)?;
        Ok(())
    }

    pub fn set_picked(
        &mut self,
        gl: &three_d::Context,
        cylinder: &three_d::CPUMesh,
        picked: bool,
    ) -> ThreeDResult<()> {
        self.picked = picked;
        self.reconstruct_models(gl, cylinder)
    }

    pub fn objects(&self) -> &[impl three_d::Object] { &self.models }
}
