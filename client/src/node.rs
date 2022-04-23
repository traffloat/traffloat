use std::error::Error;
use std::rc::Rc;

use three_d::{GeometryMut, ThreeDResult};
use traffloat_def::node::NodeId;
use traffloat_def::{atlas, building};
use traffloat_types::geometry;
use traffloat_types::space::{Matrix, Position};

use crate::texture::MaybeTexture;
use crate::{mat, texture, Server, StdMeshes};

pub struct View {
    pub id:       NodeId,
    pub position: Position,
    pub shapes:   Vec<building::Shape>,
    pub color:    [f32; 3],
}

struct PreparedInner {
    models:  Vec<Model>,
    loading: Vec<MaybeTexture>,
}

pub struct Prepared {
    pub view: View,
    inner:    PreparedInner,
}

struct NewModelContext<'t, S: Server> {
    texture_pool: &'t texture::Pool,
    gl:           &'t three_d::Context,
    server:       &'t S,
    meshes:       &'t StdMeshes,
}

impl<'t, S: Server> Clone for NewModelContext<'t, S> {
    fn clone(&self) -> Self {
        Self {
            texture_pool: self.texture_pool,
            gl:           self.gl,
            server:       self.server,
            meshes:       self.meshes,
        }
    }
}

impl<'t, S: Server> Copy for NewModelContext<'t, S> {}

pub type Model = three_d::Model<three_d::PhysicalMaterial>;

fn new_model(
    NewModelContext { texture_pool, gl, server, meshes: _ }: NewModelContext<'_, impl Server>,
    loading: &mut Vec<MaybeTexture>,
    shape: &building::Shape,
    mesh: &three_d::CPUMesh,
    tf: Matrix,
) -> ThreeDResult<Model> {
    let path = atlas::to_path("fancy", shape.texture().spritesheet_id());

    let texture = texture_pool.request(gl, server, &path);

    if texture.borrow().is_none() {
        loading.push(Rc::clone(&texture));
    }

    let texture = {
        let texture = texture.borrow();
        match &*texture {
            Some(Ok(texture)) => Some(texture.clone()),
            _ => None,
        }
    };
    let material = three_d::PhysicalMaterial {
        albedo: three_d::Color::new(200, 200, 200, 255),
        albedo_texture: texture,
        metallic: 0.3,
        roughness: 0.6,
        ..Default::default()
    };

    let mut model = Model::new_with_material(gl, mesh, material)?;
    model.set_transformation(mat(tf));

    Ok(model)
}

fn shape_to_models(
    ctx: NewModelContext<'_, impl Server>,
    shape: &building::Shape,
    tf_base: Matrix,
    models: &mut Vec<Model>,
    loading: &mut Vec<MaybeTexture>,
) -> ThreeDResult<()> {
    let tf = tf_base * shape.transform();

    match shape.unit() {
        geometry::Unit::Cube => {
            models.push(new_model(ctx, loading, shape, &ctx.meshes.cube, tf)?);
        }
        geometry::Unit::Cylinder => {
            models.push(new_model(ctx, loading, shape, &ctx.meshes.fused_cylinder, tf)?);
        }
        geometry::Unit::Sphere => {
            models.push(new_model(ctx, loading, shape, &ctx.meshes.sphere, tf)?);
        }
    }

    Ok(())
}

fn shapes_to_inner(
    ctx: NewModelContext<'_, impl Server>,
    view: &View,
) -> ThreeDResult<PreparedInner> {
    let tf_base = Matrix::new_translation(&view.position.vector());

    let mut models = Vec::new();
    let mut loading = Vec::new();

    for shape in &view.shapes {
        shape_to_models(ctx, shape, tf_base, &mut models, &mut loading)?;
    }

    Ok(PreparedInner { models, loading })
}

impl Prepared {
    pub fn new(
        view: View,
        meshes: &StdMeshes,
        texture_pool: &texture::Pool,
        gl: &three_d::Context,
        server: &impl Server,
    ) -> Result<Self, Box<dyn Error>> {
        let inner = shapes_to_inner(NewModelContext { texture_pool, gl, server, meshes }, &view)?;

        Ok(Self { view, inner })
    }

    pub fn check_loading(
        &mut self,
        meshes: &StdMeshes,
        texture_pool: &texture::Pool,
        gl: &three_d::Context,
        server: &impl Server,
    ) -> Result<(), Box<dyn Error>> {
        for maybe in &mut self.inner.loading {
            if maybe.borrow().is_some() {
                self.inner = shapes_to_inner(
                    NewModelContext { texture_pool, gl, server, meshes },
                    &self.view,
                )?;
                break;
            }
        }

        Ok(())
    }

    pub fn models(&self) -> &[Model] { &self.inner.models }

    pub fn set_picked(&mut self, b: bool) {
        for model in &mut self.inner.models {
            model.material.albedo = if b {
                three_d::Color::new(255, 255, 255, 255)
            } else {
                three_d::Color::new(200, 200, 200, 255)
            };
        }
    }
}
