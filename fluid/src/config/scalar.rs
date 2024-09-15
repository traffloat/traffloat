use bevy::ecs::system::{Res, Resource};
use bevy::ecs::world::World;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use traffloat_base::save;

use crate::units;

/// A resource storing all available fluid types.
#[derive(Resource)]
pub struct Scalar {
    /// Transferring fluid less than this amount would not trigger container element creation.
    pub creation_threshold: units::Mass,
    /// Remaining fluid less than this amount would trigger container element deletion.
    pub deletion_threshold: units::Mass,
}

impl Default for Scalar {
    fn default() -> Self {
        Self {
            creation_threshold: units::Mass { quantity: 1e-3 },
            deletion_threshold: units::Mass { quantity: 1e-6 },
        }
    }
}

/// Save schema for scalar values.
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Save {
    /// Transferring fluid less than this amount would not trigger container element creation.
    pub creation_threshold: f32,
    /// Remaining fluid less than this amount would trigger container element deletion.
    pub deletion_threshold: f32,
}

impl save::Def for Save {
    const TYPE: &'static str = "traffloat.save.fluid.ScalarConfig";

    type Runtime = ();

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(mut writer: save::Writer<Save>, (): (), config: Res<Scalar>) {
            writer.write(
                (),
                Save {
                    creation_threshold: config.creation_threshold.quantity,
                    deletion_threshold: config.deletion_threshold.quantity,
                },
            );
        }

        save::StoreSystemFn::new(store_system)
    }

    fn loader() -> impl save::LoadOnce<Def = Self> {
        #[allow(clippy::trivially_copy_pass_by_ref, clippy::unnecessary_wraps)]
        fn loader(world: &mut World, def: Save, (): &()) -> anyhow::Result<()> {
            let mut config = world.resource_mut::<Scalar>();
            config.creation_threshold.quantity = def.creation_threshold;
            config.deletion_threshold.quantity = def.deletion_threshold;

            Ok(())
        }

        save::LoadFn::new(loader)
    }
}
