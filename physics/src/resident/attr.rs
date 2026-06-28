use std::{iter, mem};

use bevy::app;
use bevy::app::{App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::{Entity, EntityHashSet};
use bevy::ecs::message::MessageWriter;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Command, Commands, Query, Res, SystemParam};
use bevy::ecs::world::World;
use bevy::reflect::Reflect;
use enum_map::EnumMap;
use serde::{Deserialize, Serialize};
use traffloat_proto::proto;

use crate::persist::AppExt;
use crate::{CleanupAppExt, view};

mod persist;
pub use persist::Persist;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Types>();
        app.register_type::<Attributes>();
        app.register_type::<LastSentConfig>();

        app.register_persistable(Persist);

        app.init_resource::<Types>();

        app.add_systems(
            app::Update,
            broadcast_attr_type_changes_system.in_set(view::SendUpdatesSystemSet::Meta),
        );
        app.add_systems(
            app::Update,
            init_viewer_system
                .in_set(view::SendUpdatesSystemSet::Init)
                .after(super::init_viewer_system),
        );
        app.add_systems(
            app::Update,
            incr_viewer_system
                .in_set(view::SendUpdatesSystemSet::Incr)
                .after(super::incr_viewer_system),
        );
        app.add_cleanup_hook(Types::cleanup_hook);
    }
}

#[derive(Resource, Reflect, Default)]
pub struct Types {
    defs: Vec<TypeDef>,

    #[reflect(ignore, default)]
    pub niches: EnumMap<Niche, Option<TypeId>>,

    generation: TypesGeneration,
}

impl Types {
    fn push(&mut self, def: TypeDef) -> TypeId {
        let id = TypeId(u32::try_from(self.defs.len()).expect("too many types"));
        self.defs.push(def);
        self.generation.0 = self.generation.0.strict_add(1);
        id
    }

    pub fn get(&self, ty: TypeId) -> &TypeDef {
        self.defs
            .get(usize::try_from(ty.0).expect("u32 <= usize on all supported targets"))
            .expect("invalid type ID")
    }

    pub fn iter(&self) -> impl Iterator<Item = (TypeId, &TypeDef)> {
        self.defs
            .iter()
            .enumerate()
            .map(|(i, def)| (TypeId(u32::try_from(i).expect("too many attribute types")), def))
    }

    pub fn len(&self) -> usize { self.defs.len() }

    pub fn is_empty(&self) -> bool { self.defs.is_empty() }

    pub fn cleanup_hook(world: &mut World) {
        let mut types = world.resource_mut::<Types>();
        types.defs.clear();
        for niche in types.niches.values_mut() {
            *niche = None;
        }
        types.generation.0 = 0;
    }
}

/// Identifies an attribute type, indexes [`Types::types`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Reflect,
)]
pub struct TypeId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TypeDef {
    pub name:          String,
    pub default_value: f32,
    #[reflect(ignore, default)]
    pub visibility:    EnumMap<view::SubscriptionLevel, bool>,
}

impl TypeDef {
    fn subscribed_by(&self, sub: view::SubscriptionLevel) -> bool { self.visibility[sub] }
}

/// Indicates that the attribute has special semantics.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect, enum_map::Enum)]
pub enum Niche {
    /// The attribute value should be treated literally as the volume ocucpied.
    Volume,
    /// When the attribute value is zero or negative,
    /// death mechanism for the resident is triggered.
    Hitpoints,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Reflect)]
struct TypesGeneration(u32);

#[derive(Component, Reflect)]
#[require(LastSentAttributes)]
pub struct Attributes {
    #[reflect(ignore, default)]
    pub values: Box<[f32]>,
}

impl Attributes {
    pub fn iter(&self) -> impl Iterator<Item = (TypeId, f32)> + '_ {
        self.values
            .iter()
            .enumerate()
            .map(|(ty, &value)| (TypeId(u32::try_from(ty).expect("checked during push")), value))
    }
}

pub struct AddTypeCommand {
    def:    TypeDef,
    niches: Vec<Niche>,
}

impl AddTypeCommand {
    pub fn new(def: TypeDef) -> Self { Self { def, niches: Vec::new() } }

    #[must_use]
    pub fn with_niche(mut self, niche: Niche) -> Self {
        self.niches.push(niche);
        self
    }
}

impl Command for AddTypeCommand {
    type Out = ();

    fn apply(self, world: &mut World) {
        let default_value = self.def.default_value;

        {
            let mut types = world.resource_mut::<Types>();
            let ty = types.push(self.def);

            for niche in self.niches {
                types.niches[niche] = Some(ty);
            }
        }

        for mut attributes in world.query::<&mut Attributes>().query_mut(world) {
            let new_box: Box<[f32]> =
                mem::take(&mut attributes.values).into_iter().chain([default_value]).collect();
            attributes.values = new_box;
        }
    }
}

/// Component on residents, indicating the attributes last broadcast normally.
#[derive(Component, Default)]
struct LastSentAttributes(Option<Box<[f32]>>);

fn init_viewer_system(
    mut writer: MessageWriter<view::SentUpdate>,
    resident_query: Query<(&LastSentAttributes, &view::Viewable)>,
    types: Res<Types>,
) {
    for (last_sent, viewable) in resident_query {
        writer.write_batch(viewable.broadcast_new_or_changed(|_, new| {
            // if last_sent is none, the data will be sent in `incr_viewer_system` anyway,
            // so we don't need to send in this system
            last_sent.0.as_ref().map(|attrs| {
                proto::UpdateResidentAttributesFull {
                    id:    viewable.id,
                    sub:   new.to_proto_subscribed_by(),
                    attrs: attrs
                        .iter()
                        .enumerate()
                        .map(|(ty, &value)| {
                            (TypeId(u32::try_from(ty).expect("checked during push")), value)
                        })
                        .filter(|&(ty, _)| types.get(ty).subscribed_by(new))
                        .map(|(_, value)| value)
                        .collect(),
                }
                .into()
            })
        }));
    }
}

fn incr_viewer_system(
    mut throttle: view::BroadcastThrottle,
    broadcast_type_params: BroadcastAttrTypeChangesParams,
    resident_query: Query<(&Attributes, &mut LastSentAttributes, &view::Viewable)>,
    types: Res<Types>,
    mut writer: MessageWriter<view::SentUpdate>,
) {
    if !throttle.should_run() {
        return;
    }

    for (attributes, mut last_sent, viewable) in resident_query {
        match last_sent.0 {
            None => {
                // attributes of this resident was never broadcast to anyone before
                writer.write_batch(viewable.broadcast_update(|level| {
                    Some(
                        proto::UpdateResidentAttributesFull {
                            id:    viewable.id,
                            sub:   level.to_proto_subscribed_by(),
                            attrs: attributes
                                .iter()
                                .filter(|&(ty, _)| types.get(ty).subscribed_by(level))
                                .map(|(_, value)| value)
                                .collect(),
                        }
                        .into(),
                    )
                }));
                last_sent.0 = Some(attributes.values.clone());
            }
            Some(ref last_values) if *last_values != attributes.values => {
                // this resident was broadcast before, but with stale values
                writer.write_batch(viewable.broadcast_update(|level| {
                    let attrs = last_values
                        .iter()
                        .zip(attributes.iter())
                        .filter(
                            #[expect(clippy::float_cmp, reason = "best-effort resend reduction")]
                            |&(&last, (ty, value))| {
                                types.get(ty).subscribed_by(level) && last != value
                            },
                        )
                        .map(|(last, (ty, value))| (ty.0, value))
                        .collect();
                    Some(proto::UpdateResidentAttributesPartial { id: viewable.id, attrs }.into())
                }));
                last_sent.0 = Some(attributes.values.clone());
            }
            Some(ref last_values) => {
                // this resident was broadcast before, and values are unchanged.
            }
        }
    }
}

/// Component on viewers, indicating the subscription config the last time they received `SetResidentAttrType`.
#[derive(Debug, Clone, Copy, Component, Default, Reflect)]
struct LastSentConfig(TypesGeneration);

#[derive(SystemParam)]
struct BroadcastAttrTypeChangesParams<'w, 's> {
    viewer_query: Query<'w, 's, (Entity, Option<&'static LastSentConfig>)>,
    types:        Res<'w, Types>,
}

fn broadcast_attr_type_changes_system(
    params: BroadcastAttrTypeChangesParams,
    mut commands: Commands,
    mut writer: MessageWriter<view::SentUpdate>,
) {
    writer.write_batch(broadcast_attr_type_changes(params, &mut commands));
}

fn broadcast_attr_type_changes(
    params: BroadcastAttrTypeChangesParams,
    commands: &mut Commands,
) -> impl Iterator<Item = view::SentUpdate> {
    let viewers: EntityHashSet = params
        .viewer_query
        .into_iter()
        .filter(|(entity, last)| last.map(|c| c.0) != Some(params.types.generation))
        .map(|(entity, _)| entity)
        .collect();
    for &viewer in &viewers {
        commands.entity(viewer).insert(LastSentConfig(params.types.generation));
    }

    // TODO benchmark this function, confirm if it would be more vectorizable
    // if index list is precomputed before iteration
    let mut types: Vec<_> = params
        .types
        .defs
        .iter()
        .map(|def| proto::ResidentAttrType {
            name:       def.name.clone(),
            subscribed: def
                .visibility
                .iter()
                .filter(|&(_, &v)| v)
                .map(|(k, _)| k.to_proto_subscribed_by())
                .collect(),
            niches:     proto::ResidentAttrNiche::empty(),
        })
        .collect();
    if let Some(volume_ty) = params.types.niches[Niche::Volume] {
        types[volume_ty.0 as usize].niches |= proto::ResidentAttrNiche::SIZE;
    }
    iter::once(view::SentUpdate { viewers, body: proto::SetResidentAttrTypes { types }.into() })
}
