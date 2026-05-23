use bevy::app::{self, App, Plugin};
use bevy::asset::{self, Assets};
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Res, ResMut, SystemParam};
use bevy::math::primitives::{Circle, Rectangle};
use bevy::mesh::Mesh;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnedShapes>();
        app.add_systems(app::Startup, init_shapes_system);
    }
}

#[derive(SystemParam)]
pub struct Shapes<'w> {
    res: Res<'w, SpawnedShapes>,
}

impl Shapes<'_> {
    pub fn circle(&self) -> asset::Handle<Mesh> {
        self.res.circle.clone().expect("Circle mesh initialized at startup")
    }

    pub fn rect(&self) -> asset::Handle<Mesh> {
        self.res.rect.clone().expect("Rect mesh initialized at startup")
    }
}

#[derive(Resource, Default)]
struct SpawnedShapes {
    circle: Option<asset::Handle<Mesh>>,
    rect:   Option<asset::Handle<Mesh>>,
}

fn init_shapes_system(mut res: ResMut<SpawnedShapes>, mut meshes: ResMut<Assets<Mesh>>) {
    res.circle = Some(meshes.add(Mesh::from(Circle::new(1.0))));
    res.rect = Some(meshes.add(Mesh::from(Rectangle::new(1.0, 1.0))));
}
