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
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use traffloat_base::{debug, proto, save};
use traffloat_view::{appearance, viewable};
use typed_builder::TypedBuilder;

/// Components for a facility.
#[derive(bundle::Bundle, TypedBuilder)]
#[allow(missing_docs)]
pub struct Bundle {
    viewable: viewable::StationaryChildBundle,
    #[builder(default, setter(skip))]
    _marker:  Marker,
    #[builder(default = debug::Bundle::new("Facility"))]
    _debug:   debug::Bundle,
}

/// Marks an entity as a facility.
#[derive(Component, Default)]
pub struct Marker;

/// Save schema.
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Save {
    /// Reference to parent building.
    pub parent:     save::Id<super::Save>,
    /// Position of the facility relative to the building center.
    pub inner:      proto::Transform,
    /// Appearance of the facility.
    pub appearance: appearance::Appearance,
    /// Whether the facility is the ambient facility of its parent building.
    pub is_ambient: bool,
}

impl save::Def for Save {
    const TYPE: &'static str = "traffloat.save.Facility";

    type Runtime = Entity;

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(
            mut writer: save::Writer<Save>,
            (building_dep,): (save::StoreDepend<super::Save>,),
            (query, building_query): (
                Query<
                    (Entity, &hierarchy::Parent, &Transform, &appearance::Appearance),
                    With<Marker>,
                >,
                Query<&super::FacilityList, With<super::Marker>>,
            ),
        ) {
            writer.write_all(query.iter().map(|(entity, parent, &transform, appearance)| {
                (
                    entity,
                    Save {
                        parent:     building_dep.must_get(parent.get()),
                        inner:      transform.into(),
                        appearance: appearance.clone(),
                        is_ambient: building_query
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
            (building_dep,): &(save::LoadDepend<super::Save>,),
        ) -> anyhow::Result<Entity> {
            let sid = viewable::next_sid(world);

            let parent = building_dep.get(def.parent)?;
            let facility_bundle = Bundle::builder()
                .viewable(
                    viewable::StationaryChildBundle::builder()
                        .base(
                            viewable::BaseBundle::builder()
                                .sid(sid)
                                .appearance(def.appearance)
                                .build(),
                        )
                        .inner_transform(def.inner.into())
                        .build(),
                )
                .build();

            let id = if def.is_ambient {
                let id = {
                    let list = world
                        .get::<super::FacilityList>(parent)
                        .expect("parent building was created in the previous load step");
                    list.ambient
                };
                let mut facility = world.entity_mut(id);
                println!("ambient of {parent:?} is {id:?}");
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
                list.non_ambient.push(id);

                id
            };

            Ok(id)
        }

        save::LoadFn::new(loader)
    }
}
