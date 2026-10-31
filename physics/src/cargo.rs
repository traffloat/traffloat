use std::ops;

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, Query, Res};
use bevy::ecs::world::World;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};
use traffloat_proto::proto;

use crate::{types, view};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<TypeDef>();
        app.register_type::<ViewerSynced>();
        app.register_type::<Storage>();

        types::init::<TypeDef>(app);

        app.add_systems(
            app::Update,
            sync_types_to_viewers_system.in_set(view::SendUpdatesSystemSet::Meta),
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TypeDef {
    pub name:           String,
    pub description:    String,
    pub granule_mass:   f32,
    pub granule_volume: f32,
}

types::define_type! {
    "cargo", "cargo:type", TypeDef;
    TypeId, PersistDeps, Types, PersistTypes, TypesGeneration;
    depends {}
}

/// Amount of a cargo type in a storage.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    derive_more::Add,
    derive_more::AddAssign,
    derive_more::Sub,
    derive_more::SubAssign,
    Reflect,
)]
pub struct Granules(pub i64);

impl<T: Into<i64>> ops::Mul<T> for Granules {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output { Granules(self.0 * Into::<i64>::into(rhs)) }
}

impl Granules {
    #[expect(
        clippy::cast_precision_loss,
        reason = "div_f32 is only used for rough ratio calculation"
    )]
    pub fn div_f32(self, other: Self) -> f32 { self.0 as f32 / other.0 as f32 }
}

/// Component on viewers to track cargo type definition sync.
#[derive(Component, Reflect)]
pub struct ViewerSynced {
    num_types: usize,
}

fn sync_types_to_viewers_system(
    types: Res<Types>,
    viewers: Query<(Entity, Option<&ViewerSynced>), With<view::Viewer>>,
    mut commands: Commands,
    mut writer: MessageWriter<view::SentUpdate>,
) {
    fn def_to_proto(type_def: &TypeDef) -> proto::CargoType {
        proto::CargoType {
            name:        type_def.name.clone(),
            description: type_def.description.clone(),
            density:     type_def.granule_mass / type_def.granule_volume,
        }
    }

    for (entity, viewer) in viewers {
        if viewer.is_none_or(|v| v.num_types != types.len()) {
            commands.entity(entity).insert(ViewerSynced { num_types: types.len() });
            writer.write(view::SentUpdate {
                viewers: [entity].into(),
                body:    proto::Update::SetCargoTypes(proto::SetCargoTypes {
                    types: types.iter().map(|(_, def)| def_to_proto(def)).collect(),
                }),
            });
        }
    }
}

/// Stores information about cargo in a fluid storage.
///
/// This component must only be applied on entities that also have a fluid storage.
#[derive(Component, Reflect, Debug, Clone)]
pub struct Storage {
    /// A sparse, unsorted list of cargo types.
    ///
    /// The optimization goal assumes there are only very few cargo types per storage
    /// such that a branchless linear search is faster than any other data structure.
    typed: Vec<TypedStorage>,
}

impl Storage {
    pub fn get_type(&self, type_id: TypeId) -> Option<&TypedStorage> {
        self.typed.iter().find(|t| t.ty == type_id)
    }

    pub fn get_type_mut(&mut self, type_id: TypeId) -> Option<&mut TypedStorage> {
        self.typed.iter_mut().find(|t| t.ty == type_id)
    }

    pub fn get_type_or_insert(&mut self, type_id: TypeId) -> &mut TypedStorage {
        match self.typed.iter().position(|t| t.ty == type_id) {
            Some(idx) => &mut self.typed[idx],
            None => self.typed.push_mut(TypedStorage { ty: type_id, granules: Granules(0) }),
        }
    }

    pub fn types(&self) -> impl DoubleEndedIterator<Item = &TypedStorage> + ExactSizeIterator {
        self.typed.iter()
    }

    pub fn types_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = &mut TypedStorage> + ExactSizeIterator {
        self.typed.iter_mut()
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct TypedStorage {
    pub ty:       TypeId,
    /// Number of granules of this cargo in the storage.
    pub granules: Granules,
}
