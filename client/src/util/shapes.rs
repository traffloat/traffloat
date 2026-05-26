use bevy::app::{self, App, Plugin};
use bevy::asset::{self, Assets};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Res, ResMut, SystemParam};
use bevy::math::primitives::{Circle, Rectangle};
use bevy::math::{Quat, Vec2, Vec3};
use bevy::mesh::{Mesh, Mesh2d};
use bevy::transform::components::Transform;

use crate::scene::Zorder;

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

    pub fn square(&self) -> asset::Handle<Mesh> {
        self.res.square.clone().expect("Square mesh initialized at startup")
    }

    pub fn rect(&self, width: f32, from: Vec2, to: Vec2, zorder: Zorder) -> impl Bundle {
        let center = (from + to) / 2.0;
        let delta = to - from;
        (
            Mesh2d(self.res.square.clone().expect("Square mesh initialized at startup")),
            Transform {
                translation: center.extend(zorder.z()),
                rotation:    Quat::from_rotation_z(delta.y.atan2(delta.x)),
                scale:       Vec3::new(delta.length(), width, 1.0),
            },
        )
    }
}

#[derive(Resource, Default)]
struct SpawnedShapes {
    circle: Option<asset::Handle<Mesh>>,
    square: Option<asset::Handle<Mesh>>,
}

fn init_shapes_system(mut res: ResMut<SpawnedShapes>, mut meshes: ResMut<Assets<Mesh>>) {
    res.circle = Some(meshes.add(Mesh::from(Circle::new(1.0))));
    res.square = Some(meshes.add(Mesh::from(Rectangle::new(1.0, 1.0))));
}
