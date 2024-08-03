//! An internal structure of a building.

use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::ecs::world::World;
use bevy::hierarchy;
use bevy::hierarchy::BuildWorldChildren;
use bevy::transform::components::Transform;
use serde::{Deserialize, Serialize};
use traffloat_base::{proto, save};
use typed_builder::TypedBuilder;

/// Components for a facility.
#[derive(bundle::Bundle, TypedBuilder)]
#[allow(missing_docs)]
pub struct Bundle {
    inner_position: Transform,
    #[builder(default, setter(skip))]
    _marker:        Marker,
}

/// Marks an entity as a building.
#[derive(Component, Default)]
pub struct Marker;

/// Save schema.
#[derive(Serialize, Deserialize)]
pub struct Save {
    /// Reference to parent building.
    pub parent:         save::Id<super::Save>,
    /// Position of the facility relative to the building center.
    pub inner_position: proto::Position,
    /// Whether the facility is the ambient facility of its parent building.
    pub is_ambient:     bool,
}

impl save::Def for Save {
    const TYPE: &'static str = "traffloat.save.Building";

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(
            mut writer: save::Writer<Save>,
            (buildings,): (save::StoreDepend<super::Save>,),
            (query, buildings_query): (
                Query<(Entity, &hierarchy::Parent, &Transform), With<Marker>>,
                Query<&super::FacilityList, With<super::Marker>>,
            ),
        ) {
            writer.write_all(query.iter().map(|(entity, parent, transform)| {
                (
                    entity,
                    Save {
                        parent:         buildings.must_get(parent.get()),
                        inner_position: transform.translation.into(),
                        is_ambient:     buildings_query
                            .get(parent.get())
                            .expect("dangling parent building reference")
                            .ambient
                            == entity,
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
            (buildings,): &(save::LoadDepend<super::Save>,),
        ) -> anyhow::Result<Entity> {
            let parent = buildings.get(def.parent)?;
            let facility_bundle = Bundle::builder()
                .inner_position(Transform::from_translation(def.inner_position.into()))
                .build();

            let id = if def.is_ambient {
                let id = {
                    let list = world
                        .get::<super::FacilityList>(parent)
                        .expect("parent building was created in the previous load step");
                    list.ambient
                };
                let mut facility = world.entity_mut(id);
                facility.insert(facility_bundle);

                id
            } else {
                let id = {
                    let mut facility = world.spawn(facility_bundle);
                    facility.set_parent(parent);
                    facility.id()
                };

                let mut list = world
                    .get_mut::<super::FacilityList>(parent)
                    .expect("parent building was created in the previous load step");
                list.facilities.push(id);

                id
            };

            Ok(id)
        }

        save::LoadFn::new(loader)
    }
}
