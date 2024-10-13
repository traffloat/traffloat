use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::{Commands, Query, Resource, SystemParam};
use bevy::ecs::world::World;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use traffloat_base::{debug, save};

use crate::units;

/// Identifies a type of fluid.
///
/// Each fluid type is an entity, and `Type` is just a typed wrapper for such entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct Type(pub Entity);

/// A [`SystemParam`] to access the registered fluid types.
#[derive(SystemParam)]
pub struct Types<'w, 's>(Query<'w, 's, (Entity, &'static TypeDef)>);

impl<'w, 's> Types<'w, 's> {
    /// Get a fluid type definition by type ID.
    #[must_use]
    pub fn get(&self, ty: Type) -> &TypeDef {
        self.0.get(ty.0).expect("reference to unknown fluid type").1
    }

    /// Iterates over all known fluid types.
    pub fn iter(&self) -> impl Iterator<Item = (Type, &TypeDef)> {
        self.0.iter().map(|(ty, def)| (Type(ty), def))
    }
}

/// Registers a new fluid type and returns its type ID.
pub fn create_type(commands: &mut Commands, def: TypeDef) -> Type {
    let ty = Type(commands.spawn((def, debug::Bundle::new("FluidType"))).id());
    commands.push(move |world: &mut World| {
        world.resource_mut::<CreatedType>().0 = Some(ty);
        world.run_schedule(OnCreateType);
        world.resource_mut::<CreatedType>().0 = None;
    });
    ty
}

/// Systems in the schedule are called in non-deterministic order
/// immediately after a fluid type is created and initialized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct OnCreateType;

/// The type getting created.
///
/// Only valid in the [`OnCreateType`] schedule.
#[derive(Default, Resource)]
pub struct CreatedType(Option<Type>);

impl CreatedType {
    /// Gets the new type getting created that triggered the [`OnCreateType`] schedule.
    #[must_use]
    pub fn get(&self) -> Type {
        self.0.expect("CreatedType can only be used from OnCreateType systems")
    }
}

/// Defines the properties of a fluid.
#[derive(Clone, Serialize, Deserialize, JsonSchema, Component)]
pub struct TypeDef {
    /// Viscosity coefficient.
    ///
    /// Viscosity is inversely proportional to flow rate in fluid flow
    /// and diffusion rate in diffusion respectively.
    pub viscosity: units::Viscosity,

    /// The specific volume (reciprocal of density) of the fluid during vacuum phase.
    pub vacuum_specific_volume: units::SpecificVolume,

    /// The pressure above which the fluid exhibits saturation phase properties.
    pub critical_pressure: units::Pressure,

    /// The amplitification coefficient for saturated fluids.
    pub saturation_gamma: f32,
}

/// Save schema for scalar values.
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Save {
    #[serde(flatten)]
    def: TypeDef,
}

impl save::Def for Save {
    const TYPE: &'static str = "traffloat.save.fluid.Type";

    type Runtime = Type;

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(mut writer: save::Writer<Save>, (): (), query: Query<(Entity, &TypeDef)>) {
            writer.write_all(query.iter().map(|(ty, def)| (Type(ty), Save { def: def.clone() })));
        }

        save::StoreSystemFn::new(store_system)
    }

    fn loader() -> impl save::LoadOnce<Def = Self> {
        #[allow(clippy::trivially_copy_pass_by_ref, clippy::unnecessary_wraps)]
        fn loader(world: &mut World, def: Save, (): &()) -> anyhow::Result<Type> {
            let ty = create_type(&mut world.commands(), def.def);
            Ok(ty)
        }

        save::LoadFn::new(loader)
    }
}
