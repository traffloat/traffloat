use std::ops;

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::{IntoScheduleConfigs, SystemSet};
use bevy::ecs::system::{Command, Commands, EntityCommand, Query, Res};
use bevy::ecs::world::{EntityWorldMut, World};
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};
use traffloat_proto::proto;

use crate::persist::AppExt;
use crate::{CleanupAppExt, view};

pub mod persist;

mod persist_type;
pub use persist_type::Persist as PersistTypes;
mod transfer;

/// A constant equivalent to the ideal gas constant, used for pressure calculation.
pub const PRESSURE_COEFFICIENT: f32 = 1.0 / 128.0;

/// A constant to adjust the base diffusion rate.
pub const DIFFUSION_COEFFICIENT: f32 = 1.0 / 8192.0;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Types>();
        app.register_type::<TypeDef>();
        app.register_type::<Storage>();
        app.register_type::<Edge>();
        app.register_type::<AlphaOfEdgeList>();
        app.register_type::<EdgeAlpha>();
        app.register_type::<BetaOfEdgeList>();
        app.register_type::<EdgeBeta>();
        app.register_type::<Sensor>();
        app.register_type::<ViewerSynced>();

        app.register_persistable(PersistTypes);

        app.init_resource::<Conf>();
        app.init_resource::<Types>();

        app.add_systems(app::FixedUpdate, transfer::transfer_system.in_set(TransferSystemSet));
        app.add_systems(
            app::Update,
            sync_types_to_viewers_system.in_set(view::SendUpdatesSystemSet::Meta),
        );
        app.add_cleanup_hook(Types::cleanup_hook);
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransferSystemSet;

#[derive(Resource)]
pub struct Conf {
    pub transfer_timestep: u32,
}

impl Default for Conf {
    fn default() -> Self { Self { transfer_timestep: 16 } }
}

/// Amount of fluid substance,
/// in an arbitrary unit equivalent to moles.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    derive_more::Add,
    derive_more::AddAssign,
    derive_more::Sub,
    derive_more::SubAssign,
    Reflect,
)]
pub struct Moles(pub f32);

impl ops::Mul<f32> for Moles {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self { Self(self.0 * rhs) }
}

/// Amount of heat energy,
/// in an arbitrary unit equivalent to joules.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    derive_more::Add,
    derive_more::AddAssign,
    derive_more::Sub,
    derive_more::SubAssign,
    Reflect,
)]
pub struct Energy(pub f32);

impl ops::Mul<f32> for Energy {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self { Self(self.0 * rhs) }
}

/// Identifies a fluid type, indexes [`Types::types`].
///
/// Unlike [`Entity`], this is a stable identifier that is exactly restored
/// across network sync and persistence.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Reflect,
)]
pub struct TypeId(pub u32);

#[derive(Resource, Reflect, Default)]
pub struct Types {
    pub types: Vec<TypeDef>,
}

impl Types {
    #[must_use]
    pub fn get(&self, ty: TypeId) -> &TypeDef {
        self.types.get(ty.0 as usize).expect("invalid fluid type reference created")
    }

    pub fn push(&mut self, type_def: TypeDef) -> TypeId {
        let id = u32::try_from(self.types.len()).expect("too many fluid types");
        self.types.push(type_def);
        TypeId(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (TypeId, &TypeDef)> {
        self.types
            .iter()
            .enumerate()
            .map(|(i, t)| (TypeId(u32::try_from(i).expect("too many fluid types")), t))
    }

    fn cleanup_hook(world: &mut World) { world.resource_mut::<Types>().types.clear(); }
}

pub struct AddTypeCommand {
    pub type_def: TypeDef,
}

impl Command for AddTypeCommand {
    fn apply(self, world: &mut World) {
        let mut types = world.resource_mut::<Types>();
        types.push(self.type_def);
        let num_types = types.types.len();

        for mut storage in world.query::<&mut Storage>().iter_mut(world) {
            let new_typed =
                (0..num_types).map(|i| storage.types.get(i).copied().unwrap_or_default()).collect();
            storage.types = new_typed;
        }

        for mut edge in world.query::<&mut Edge>().iter_mut(world) {
            let new_typed = (0..num_types)
                .map(|i| edge.last_typed_transfer.get(i).cloned().unwrap_or_default())
                .collect();
            edge.last_typed_transfer = new_typed;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TypeDef {
    pub name:                 String,
    /// Heat energy per mole.
    pub molar_heat_capacity:  f32,
    /// Mass per mole.
    pub molar_density:        f32,
    /// Multiplier to advection rate.
    ///
    /// Must not exceed 1.0.
    pub advective_fluidity:   f32,
    /// Multiplier to diffusion rate.
    ///
    /// Must not exceed 1.0.
    pub diffusive_fluidity:   f32,
    /// Multiplier to heat transfer through conduction.
    pub thermal_conductivity: f32,
    /// Base multiplier for transmitted color.
    pub optical_extinction:   [f32; 3],
    // /// Base multiplier for emitted color.
    // pub optical_emission:  [f32; 3],
}

#[derive(Component, Reflect, Debug, Clone)]
#[require(Sensor)]
pub struct Storage {
    /// Volume provided by the storage.
    ///
    /// This may change over time subject to displacement,
    /// e.g. dumping cargo into a storage.
    ///
    /// May be mutated by modules defining a storage.
    pub volume: f32,
    /// Length of the storage to compute light absorption rate.
    pub length: f32,

    /// Heat energy in the storage.
    pub heat:  Energy,
    /// Must always be sorted by type.
    // TODO benchmark possible alternative representations:
    // 1. use smallvec
    // 2. use dynamic components for each type
    pub types: Vec<TypedStorage>,

    // derived quantities
    /// Force per unit area exerted by the mixture.
    pub pressure:    f32,
    /// Absolute temperature.
    pub temperature: f32,
    /// Mass of fluid in this storage, used for force calculation in other modules.
    pub mass:        f32,
    /// Total moles of fluid in this storage.
    pub moles:       Moles,

    pub optical_extinction: [f32; 3],
    // pub optical_emission: [f32; 3],
    /// RGBA color of the container.
    pub rgba:               [f32; 4],
}

impl Storage {
    #[must_use]
    pub fn vacuum(num_types: usize, volume: f32, length: f32) -> Self {
        Self {
            volume,
            length,
            heat: Energy(0.0),
            types: (0..num_types).map(|_| TypedStorage::default()).collect(),
            pressure: 0.0,
            temperature: 0.0,
            mass: 0.0,
            moles: Moles(0.0),
            optical_extinction: [0.0; 3],
            // optical_emission: [0.0; 3],
            rgba: [0.0; 4],
        }
    }

    #[must_use]
    pub fn with_heat(mut self, heat: Energy) -> Self {
        self.heat = heat;
        self
    }

    pub fn set_heat(&mut self, heat: Energy) { self.heat = heat; }

    #[must_use]
    pub fn with_fluid(mut self, ty: TypeId, moles: f32) -> Self {
        self.set_fluid(ty, Moles(moles));
        self
    }

    pub fn set_fluid(&mut self, ty: TypeId, moles: Moles) {
        // for performance reasons,
        // we are not going to update proportion and molar conc until the next tick.
        // This is expected to have negligible impact
        // since it only affects flow rate multiplier computation for one tick.

        self.get_type_mut(ty).moles = moles;
    }

    /// Computes total heat capacity based on derived quantities.
    #[must_use]
    pub fn derived_total_heat_capacity(&self) -> f32 {
        if self.heat.0 == 0.0 || self.temperature == 0.0 {
            return 0.0;
        }
        self.heat.0 / self.temperature
    }

    #[must_use]
    pub fn get_type(&self, ty: TypeId) -> &TypedStorage {
        self.types
            .get(ty.0 as usize)
            .expect("all fluid storages must be resized after adding fluid storages")
    }

    #[must_use]
    pub fn get_type_mut(&mut self, ty: TypeId) -> &mut TypedStorage {
        self.types
            .get_mut(ty.0 as usize)
            .expect("all fluid storages must be resized after adding fluid storages")
    }

    pub fn types(&self) -> impl Iterator<Item = (TypeId, &TypedStorage)> {
        self.types
            .iter()
            .enumerate()
            .map(|(i, t)| (TypeId(u32::try_from(i).expect("too many types in storage")), t))
    }

    pub fn types_mut(&mut self) -> impl Iterator<Item = (TypeId, &mut TypedStorage)> {
        self.types
            .iter_mut()
            .enumerate()
            .map(|(i, t)| (TypeId(u32::try_from(i).expect("too many types in storage")), t))
    }

    pub fn to_proto_normal(&self, sensor: &Sensor) -> proto::FluidStorageDetail {
        proto::FluidStorageDetail {
            volume:      self.volume,
            pressure:    sensor.pressure.then_some(self.pressure),
            temperature: sensor.temperature.then_some(self.temperature),
            types:       None,
        }
    }

    pub fn to_proto_debug(&self) -> proto::FluidStorageDetail {
        proto::FluidStorageDetail {
            volume:      self.volume,
            pressure:    Some(self.pressure),
            temperature: Some(self.temperature),
            types:       Some(self.types.iter().map(|typed| typed.moles.0).collect()),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Reflect)]
pub struct TypedStorage {
    /// Amount of this type in this storage, in moles.
    pub moles: Moles,

    // derived quantities
    /// Proportion of volume in this storage occupied by this type, between 0 and 1.
    pub proportion: f32,
    /// Moles per unit volume of this type in this storage.
    pub molar_conc: f32,
}

pub struct AddStorageCommand {
    pub volume:         f32,
    pub optical_length: f32,
}

impl EntityCommand for AddStorageCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let num_types = entity.world().resource::<Types>().types.len();
        entity.insert(Storage::vacuum(num_types, self.volume, self.optical_length));
    }
}

pub struct SetTemperatureCommand {
    pub temperature: f32,
}

impl EntityCommand for SetTemperatureCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let storage = try_log!(entity.get::<Storage>(), expect "SetTemperatureCommand must be applied on fluid storage entities" or return);
        let types = entity.world().resource::<Types>();
        let heat = self.temperature
            * storage
                .types()
                .map(|(type_id, typed)| {
                    let type_def = types.get(type_id);
                    typed.moles.0 * type_def.molar_heat_capacity
                })
                .sum::<f32>();

        let mut storage = entity.get_mut::<Storage>().expect("checked at function start");
        storage.heat = Energy(heat);
    }
}

/// The connection between two storages, through which fluid can transfer.
#[derive(Component, Reflect, Debug, Clone)]
pub struct Edge {
    /// Reciprocal of resistance to transfer.
    ///
    /// The reciprocal is proportional to flow rate.
    /// Physically resembles the reciprocal of the length of a pipe.
    pub resistance_recip: f32,
    /// Cross-sectional area of the connection.
    ///
    /// Directly proportional to flow rate and heat conduction rate.
    pub area:             f32,

    /// Additional force per unit area from alpha to beta, e.g. from a pump or valve.
    pub force_atob: f32,

    // Temporary states.
    /// Heat transferred from alpha to beta in the last transfer step.
    last_heat:           Energy,
    /// Must always be sorted by type.
    last_typed_transfer: Vec<TypedTransfer>,
}

impl Edge {
    #[must_use]
    pub fn new(num_types: usize, resistance_recip: f32, area: f32) -> Self {
        Self {
            resistance_recip,
            area,
            force_atob: 0.0,
            last_heat: Energy(0.0),
            last_typed_transfer: (0..num_types).map(|_| TypedTransfer::default()).collect(),
        }
    }

    #[must_use]
    pub fn with_force_atob(mut self, force: f32) -> Self {
        self.force_atob = force;
        self
    }

    #[must_use]
    pub fn with_force_btoa(mut self, force: f32) -> Self {
        self.force_atob = -force;
        self
    }
}

#[derive(Debug, Clone, Default, Reflect)]
struct TypedTransfer {
    atob_transfer: Moles,
}

pub struct AddEdgeCommand {
    pub resistance_recip: f32,
    pub area:             f32,

    pub alpha: Entity,
    pub beta:  Entity,
}

impl EntityCommand for AddEdgeCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let num_types = entity.world().resource::<Types>().types.len();
        entity.insert((
            Edge::new(num_types, self.resistance_recip, self.area),
            EdgeAlpha(self.alpha),
            EdgeBeta(self.beta),
        ));
    }
}

#[derive(Component, Reflect)]
#[relationship(relationship_target = AlphaOfEdgeList)]
pub struct EdgeAlpha(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship = EdgeAlpha, linked_spawn)]
pub struct AlphaOfEdgeList(Vec<Entity>);

#[derive(Component, Reflect)]
#[relationship(relationship_target = BetaOfEdgeList)]
pub struct EdgeBeta(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship = EdgeBeta, linked_spawn)]
pub struct BetaOfEdgeList(Vec<Entity>);

/// Allows normal viewers to receive metrics about a storage.
/// Component on storages.
#[derive(Component, Reflect, Default)]
pub struct Sensor {
    pub pressure:    bool,
    pub temperature: bool,
}

/// Component on viewers to track fluid type definition sync.
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
    for (entity, viewer) in viewers {
        if viewer.is_none_or(|v| v.num_types != types.types.len()) {
            commands.entity(entity).insert(ViewerSynced { num_types: types.types.len() });
            writer.write(view::SentUpdate {
                viewers: [entity].into(),
                body:    proto::Update::SetFluidTypes(proto::SetFluidTypes {
                    types: types
                        .types
                        .iter()
                        .map(|type_def| proto::FluidType { name: type_def.name.clone() })
                        .collect(),
                }),
            });
        }
    }
}

#[cfg(test)]
mod tests;
