//! A container element corresponds to an active fluid type in a container.
//!
//! A container element is created when an adjacent pipe wants to transfer
//! a new fluid type into this container.

use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::ecs::world::World;
use bevy::hierarchy::{self, BuildWorldChildren};
use derive_more::From;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use traffloat_base::save;
use typed_builder::TypedBuilder;

use crate::{config, units};

/// Components to construct a container element.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    ty:      config::Type,
    #[builder(setter(into))]
    mass:    Mass,
    #[builder(default = Volume { volume: <_>::default() })]
    volume:  Volume,
    #[builder(default, setter(skip))]
    _marker: Marker,
}

/// Marks an entity as a container element.
#[derive(Component, Default)]
pub struct Marker;

/// Mass of a fluid type in a container.
#[derive(Component, From)]
pub struct Mass {
    /// Typed mass value.
    pub mass: units::Mass,
}

/// The current volume occupied by a fluid type in a container.
#[derive(Component, From)]
pub struct Volume {
    /// Typed volume value.
    pub volume: units::Volume,
}

/// Save schema.
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Save {
    /// Reference to parent container.
    pub parent: save::Id<super::Save>,
    /// Type of fluid for this element.
    pub ty:     save::Id<config::SaveType>,
    /// The max of fluids in this container.
    pub mass:   units::Mass,
}

impl save::Def for Save {
    const TYPE: &'static str = "traffloat.save.fluid.ContainerElement";

    type Runtime = Entity;

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(
            mut writer: save::Writer<Save>,
            (container_dep, type_dep): (
                save::StoreDepend<super::Save>,
                save::StoreDepend<config::SaveType>,
            ),
            query: Query<(Entity, &hierarchy::Parent, &config::Type, &Mass), With<Marker>>,
        ) {
            writer.write_all(query.iter().map(|(entity, parent, &ty, mass)| {
                (
                    entity,
                    Save {
                        parent: container_dep.must_get(parent.get()),
                        ty:     type_dep.must_get(ty),
                        mass:   mass.mass,
                    },
                )
            }));
        }

        save::StoreSystemFn::new(store_system)
    }

    fn loader() -> impl save::LoadOnce<Def = Self> {
        #[allow(clippy::trivially_copy_pass_by_ref, clippy::unnecessary_wraps)]
        fn loader(
            world: &mut World,
            def: Save,
            (container_dep, type_dep): &(
                save::LoadDepend<super::Save>,
                save::LoadDepend<config::SaveType>,
            ),
        ) -> anyhow::Result<Entity> {
            let bundle = Bundle::builder().ty(type_dep.get(def.ty)?).mass(def.mass).build();

            let mut container = world.spawn(bundle);
            container.set_parent(container_dep.get(def.parent)?);
            Ok(container.id())
        }

        save::LoadFn::new(loader)
    }
}
