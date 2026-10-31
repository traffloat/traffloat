use bevy::app::{App, Plugin};
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{ResMut, SystemParam};
use traffloat_proto::proto;

use crate::{HandlerClass, UpdateHandler};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.init_resource::<FluidTypes>();
        app.init_resource::<CargoTypes>();
    }
}

#[derive(Resource)]
pub struct Types<T>(pub Vec<T>);

impl<T> Default for Types<T> {
    fn default() -> Self { Self(Vec::new()) }
}

pub type FluidTypes = Types<FluidType>;

pub struct FluidType {
    pub name:        String,
    pub description: String,
}

pub type CargoTypes = Types<CargoType>;

pub struct CargoType {
    pub name:        String,
    pub description: String,
    pub density:     f32,
}

#[derive(SystemParam)]
pub struct SetFluidTypesParams<'w> {
    fluids: ResMut<'w, FluidTypes>,
}

impl UpdateHandler for SetFluidTypesParams<'_> {
    type Update = proto::SetFluidTypes;

    fn classify(_update: &Self::Update) -> HandlerClass { HandlerClass::Meta }

    fn handle(&mut self, update: &proto::SetFluidTypes) {
        self.fluids.0 = update
            .types
            .iter()
            .map(|t| FluidType { name: t.name.clone(), description: t.description.clone() })
            .collect();
    }
}

#[derive(SystemParam)]
pub struct SetCargoTypesParams<'w> {
    cargos: ResMut<'w, CargoTypes>,
}

impl UpdateHandler for SetCargoTypesParams<'_> {
    type Update = proto::SetCargoTypes;

    fn classify(_update: &Self::Update) -> HandlerClass { HandlerClass::Meta }

    fn handle(&mut self, update: &proto::SetCargoTypes) {
        self.cargos.0 = update
            .types
            .iter()
            .map(|t| CargoType {
                name:        t.name.clone(),
                description: t.description.clone(),
                density:     t.density,
            })
            .collect();
    }
}
