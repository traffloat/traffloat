use std::mem;

use bevy::app;
use bevy::app::{App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::{Entity, EntityHashSet};
use bevy::ecs::message::MessageWriter;
use bevy::ecs::query::Changed;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Command, Commands, Query, Res};
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

        app.add_systems(app::Update, incr_viewer_system.in_set(view::SendUpdatesSystemSet::Incr));
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

    pub fn types(&self) -> &[TypeDef] { &self.types }

    pub fn get(&self, ty: TypeId) -> &TypeDef {
        self.types
            .get(usize::try_from(ty.0).expect("u32 <= usize on all supported targets"))
            .expect("invalid type ID")
    }
}

/// Identifies an attribute type, indexes [`Types::types`].
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

    fn subscribed_by_level(&self, sub: view::SubscriptionLevel) -> bool {
        self.visibility[match sub {
            view::SubscriptionLevel::Basic => view::SubscriptionConfig::Basic,
            view::SubscriptionLevel::Full => view::SubscriptionConfig::Full,
        }]
    }
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

impl Attributes {
    pub fn iter(&self) -> impl Iterator<Item = (TypeId, f32)> + '_ {
        self.values.iter().enumerate().map(|(ty, &value)| (TypeId(ty as u32), value))
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

impl Command<TypeId> for AddTypeCommand {
    fn apply(self, world: &mut World) -> TypeId {
        let default_value = self.def.default_value;

        let ty = {
            let mut types = world.resource_mut::<Types>();
            let ty = types.push(self.def);

            for niche in self.niches {
                types.niches[niche] = Some(ty);
            }

            ty
        };

        for mut attributes in world.query::<&mut Attributes>().query_mut(world) {
            let new_box: Box<[f32]> =
                mem::take(&mut attributes.values).into_iter().chain([default_value]).collect();
            attributes.values = new_box;
        }

        ty
    }
}

/// Component on residents, indicating the attributes last broadcast normally.
#[derive(Component, Default)]
struct LastSentAttributes(Option<Box<[f32]>>);

/// Component on viewers, indicating the subscription config the last time they received `SetResidentAttrType`.
#[derive(Component, Default)]
struct LastSentConfig(view::SubscriptionConfig);

fn incr_viewer_system(
    mut throttle: view::BroadcastThrottle,
    resident_query: Query<(&Attributes, &mut LastSentAttributes, &view::Viewable)>,
    viewer_query: Query<
        (Entity, &view::Viewer, Option<&mut LastSentConfig>),
        Changed<view::Viewer>,
    >,
    types: Res<Types>,
    mut writer: MessageWriter<view::SentUpdate>,
    mut commands: Commands,
) {
    if !throttle.should_run() {
        return;
    }

    let mut subs = EnumMap::<view::SubscriptionConfig, EntityHashSet>::default();
    for (entity, viewer, last_config) in viewer_query {
        if last_config.as_deref().map(|c| &c.0) != Some(&viewer.config) {
            subs[viewer.config].insert(entity);
            commands.entity(entity).insert(LastSentConfig(viewer.config));
        }
    }
    writer.write_batch(subs.into_iter().filter(|(_, viewers)| !viewers.is_empty()).map(
        |(sub, viewers)| {
            let mut sent_types: Vec<_> = types
                .types
                .iter()
                .map(|def| proto::ResidentAttrType {
                    name:       def.name.clone(),
                    subscribed: def.subscribed_by(sub),
                    niches:     proto::ResidentAttrNiche::empty(),
                })
                .collect();
            if let Some(volume_ty) = types.niches[Niche::Volume] {
                sent_types[volume_ty.0 as usize].niches |= proto::ResidentAttrNiche::SIZE;
            }
            view::SentUpdate {
                viewers,
                body: proto::SetResidentAttrTypes { types: sent_types }.into(),
            }
        },
    ));

    for (attributes, mut last_sent, viewable) in resident_query {
        match last_sent.0 {
            None => {
                // this resident was never broadcast to anyone before
                writer.write_batch(viewable.broadcast_update(|level| {
                    Some(
                        proto::UpdateResidentAttributesFull {
                            id:    viewable.id,
                            attrs: attributes
                                .iter()
                                .filter(|&(ty, _)| types.get(ty).subscribed_by_level(level))
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
                                types.get(ty).subscribed_by_level(level) && last != value
                            },
                        )
                        .map(|(last, (ty, value))| (ty.0, value))
                        .collect();
                    Some(proto::UpdateResidentAttributesPartial { id: viewable.id, attrs }.into())
                }));
                last_sent.0 = Some(attributes.values.clone());
            }
            Some(ref last_values) => {
                // this resident was broadcast before, and values are unchanged,
                // but we still need to resend to new viewers and viewers who have changed
                // subscription level.

                writer.write_batch(viewable.broadcast_new_by_level(|level| {
                    Some(
                        proto::UpdateResidentAttributesFull {
                            id:    viewable.id,
                            attrs: attributes
                                .iter()
                                .filter(|&(ty, _)| types.get(ty).subscribed_by_level(level))
                                .map(|(_, value)| value)
                                .collect(),
                        }
                        .into(),
                    )
                }));
            }
        }
    }
}
