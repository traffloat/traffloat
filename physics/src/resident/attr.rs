use std::mem;

use bevy::app::{App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::{Entity, EntityHashSet};
use bevy::ecs::message::MessageWriter;
use bevy::ecs::query::Changed;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Command, Query, Res};
use bevy::ecs::world::World;
use bevy::reflect::Reflect;
use enum_map::EnumMap;
use traffloat_proto::proto;

use crate::view;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Types>();
        app.register_type::<Attributes>();

        app.init_resource::<Types>();
    }
}

#[derive(Resource, Reflect, Default)]
pub struct Types {
    types: Vec<TypeDef>,

    #[reflect(ignore, default)]
    pub niches: EnumMap<Niche, Option<TypeId>>,
}

impl Types {
    fn push(&mut self, def: TypeDef) -> TypeId {
        let id = TypeId(u32::try_from(self.types.len()).expect("too many types"));
        self.types.push(def);
        id
    }
}

/// Identifies a fluid type, indexes [`Types::types`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub struct TypeId(pub u32);

#[derive(Reflect)]
pub struct TypeDef {
    pub name:          String,
    pub default_value: f32,
    #[reflect(ignore, default)]
    pub visibility:    EnumMap<view::SubscriptionConfig, bool>,
}

impl TypeDef {
    fn subscribed_by(&self, sub: view::SubscriptionConfig) -> bool { self.visibility[sub] }
}

/// Indicates that the attribute has special semantics.
#[derive(Reflect, enum_map::Enum)]
pub enum Niche {
    /// Represents the size
    Volume,
    Health,
}

#[derive(Component, Reflect)]
#[require(LastSentAttributes)]
pub struct Attributes {
    #[reflect(ignore, default)]
    pub values: Box<[f32]>,
}

pub struct AddTypeCommand(pub TypeDef);

impl Command for AddTypeCommand {
    fn apply(self, world: &mut World) {
        let default_value = self.0.default_value;

        {
            let mut types = world.resource_mut::<Types>();
            types.push(self.0);
        }

        for mut attributes in world.query::<&mut Attributes>().query_mut(world) {
            let new_box: Box<[f32]> =
                mem::take(&mut attributes.values).into_iter().chain([default_value]).collect();
            attributes.values = new_box;
        }
    }
}

#[derive(Component, Default)]
struct LastSentAttributes(Option<Box<[f32]>>);

fn incr_viewer_system(
    mut throttle: view::BroadcastThrottle,
    resident_query: Query<(&Attributes, &mut LastSentAttributes, &view::Viewable)>,
    viewer_query: Query<(Entity, &view::Viewer), Changed<view::Viewer>>,
    types: Res<Types>,
    mut writer: MessageWriter<view::SentUpdate>,
) {
    if !throttle.should_run() {
        return;
    }

    let mut subs = EnumMap::<view::SubscriptionConfig, EntityHashSet>::default();
    for (entity, viewer) in viewer_query {
        subs[viewer.config].insert(entity);
    }
    writer.write_batch(subs.into_iter().filter(|(_, viewers)| !viewers.is_empty()).map(
        |(sub, viewers)| {
            view::SentUpdate {
                viewers,
                body: proto::SetResidentAttrTypes {
                    types: types
                        .types
                        .iter()
                        .map(|def| proto::ResidentAttrType {
                            name:       def.name.clone(),
                            subscribed: def.subscribed_by(sub),
                        })
                        .collect(),
                }
                .into(),
            }
        },
    ));

    for (attributes, mut last_sent, viewable) in resident_query {
        match last_sent.0 {
            None => {
                writer.write_batch(viewable.broadcast_update(|level| {
                    Some(
                        proto::UpdateResidentAttributesFull {
                            id:    viewable.id,
                            attrs: attributes.values.clone().into(),
                        }
                        .into(),
                    )
                }));
            }
            Some(ref last_values) if *last_values != attributes.values => {
                writer.write_batch(viewable.broadcast_update(|level| {
                    Some(
                        proto::UpdateResidentAttributesPartial {
                            id:    viewable.id,
                            attrs: last_values
                                .iter()
                                .zip(&*attributes.values)
                                .enumerate()
                                .filter(
                                    #[expect(
                                        clippy::float_cmp,
                                        reason = "best-effort resend reduction"
                                    )]
                                    |(_, (last, value))| last != value,
                                )
                                .map(|(ty, (_, &value))| {
                                    (u32::try_from(ty).expect("type must be within bounds"), value)
                                })
                                .collect(),
                        }
                        .into(),
                    )
                }));
            }
            _ => continue,
        }
        last_sent.0 = Some(attributes.values.clone());
    }
}
