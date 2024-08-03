//! Fluid definitions.

use bevy::ecs::system::Res;
use bevy::ecs::world::World;
use bevy::prelude::{Component, Resource};
use serde::{Deserialize, Serialize};
use traffloat_base::save;

use crate::units;

/// Identifies a type of fluid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct Type(pub u16);

/// A resource storing all available fluid types.
#[derive(Resource)]
pub struct Config {
    defs:                   Vec<TypeDef>,
    /// Transferring fluid less than this amount would not trigger container element creation.
    pub creation_threshold: units::Mass,
    /// Remaining fluid less than this amount would trigger container element deletion.
    pub deletion_threshold: units::Mass,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            defs:               Vec::new(),
            creation_threshold: units::Mass { quantity: 1e-3 },
            deletion_threshold: units::Mass { quantity: 1e-6 },
        }
    }
}

impl Config {
    /// Registers a new fluid type. Only for constructing test cases.
    #[allow(clippy::missing_panics_doc)]
    pub fn register_type(&mut self, def: TypeDef) -> Type {
        let ret = Type(u16::try_from(self.defs.len()).expect("too many types"));
        self.defs.push(def);
        ret
    }

    /// Gets the definition of a fluid type
    ///
    /// # Panics
    /// Panics if the fluid type does not exist.
    #[must_use]
    pub fn get_type(&self, ty: Type) -> &TypeDef {
        self.defs.get(usize::from(ty.0)).expect("reference to unknown fluid type")
    }

    /// Iterates over all fluid types.
    #[allow(clippy::missing_panics_doc)]
    pub fn iter_types(&self) -> impl Iterator<Item = (Type, &TypeDef)> {
        self.defs
            .iter()
            .enumerate()
            .map(|(index, def)| (Type(u16::try_from(index).expect("too many fluid types")), def))
    }
}

/// Defines the properties of a fluid.
#[derive(Clone, Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
pub struct SaveScalar {
    /// Transferring fluid less than this amount would not trigger container element creation.
    pub creation_threshold: f32,
    /// Remaining fluid less than this amount would trigger container element deletion.
    pub deletion_threshold: f32,
}

impl save::Def for SaveScalar {
    const TYPE: &'static str = "traffloat.save.fluid.ScalarConfig";

    type Runtime = ();

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(mut writer: save::Writer<SaveScalar>, (): (), config: Res<Config>) {
            writer.write(
                (),
                SaveScalar {
                    creation_threshold: config.creation_threshold.quantity,
                    deletion_threshold: config.deletion_threshold.quantity,
                },
            );
        }

        save::StoreSystemFn::new(store_system)
    }

    fn loader() -> impl save::LoadOnce<Def = Self> {
        #[allow(clippy::trivially_copy_pass_by_ref, clippy::unnecessary_wraps)]
        fn loader(world: &mut World, def: SaveScalar, (): &()) -> anyhow::Result<()> {
            let mut config = world.resource_mut::<Config>();
            config.creation_threshold.quantity = def.creation_threshold;
            config.deletion_threshold.quantity = def.deletion_threshold;

            Ok(())
        }

        save::LoadFn::new(loader)
    }
}

/// Save schema for scalar values.
#[derive(Serialize, Deserialize)]
pub struct SaveType {
    #[serde(flatten)]
    def: TypeDef,
}

impl save::Def for SaveType {
    const TYPE: &'static str = "traffloat.save.fluid.Type";

    type Runtime = Type;

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(mut writer: save::Writer<SaveType>, (): (), config: Res<Config>) {
            writer.write_all(
                config.iter_types().map(|(ty, def)| (ty, SaveType { def: def.clone() })),
            );
        }

        save::StoreSystemFn::new(store_system)
    }

    fn loader() -> impl save::LoadOnce<Def = Self> {
        #[allow(clippy::trivially_copy_pass_by_ref, clippy::unnecessary_wraps)]
        fn loader(world: &mut World, def: SaveType, (): &()) -> anyhow::Result<Type> {
            let mut config = world.resource_mut::<Config>();
            let ty = config.register_type(def.def);
            Ok(ty)
        }

        save::LoadFn::new(loader)
    }
}
