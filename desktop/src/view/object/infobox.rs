use bevy::app::{self, App};
use bevy::color::Color;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::{Event, EventReader, EventWriter};
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Query, Res, ResMut, Resource};
use bevy::hierarchy::{self, BuildChildren, DespawnRecursiveExt, HierarchyQueryExt};
use bevy::render::view::Visibility;
use bevy::state::state::{self};
use bevy::text::{Text, TextSection, TextStyle};
use bevy::ui::node_bundles::{NodeBundle, TextBundle};
use bevy::ui::{self, Style, UiRect};
use bevy_eventlistener::callbacks::Listener;
use bevy_eventlistener::event_listener::On;
use bevy_mod_picking::prelude::{self as pick, Pointer};
use bevy_mod_picking::PickableBundle;
use traffloat_base::debug;
use traffloat_base::partition::AppExt;
use traffloat_view::appearance::Appearance;
use traffloat_view::viewable;

use super::metrics;
use crate::view::delegate;
use crate::{view, AppState};

type Depth = u16;

const HIERARCHY_LAYERS: Depth = 3;

pub(super) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Focus { entity: None, focus_type: FocusType::Hover });
        app.add_partitioned_event::<FocusChangeEvent>();
        app.add_systems(state::OnEnter(AppState::GameView), setup);
        app.add_systems(app::Update, update_hierarchy_system);
        app.add_systems(app::Update, update_box_visibility_system);
        app.add_systems(app::Update, update_viewable_label_system.after(update_hierarchy_system));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: ui::Val::Px(200.),
                height: ui::Val::Px(300.),
                justify_self: ui::JustifySelf::End,
                align_self: ui::AlignSelf::End,
                border: UiRect::all(ui::Val::Px(5.)),
                padding: UiRect::all(ui::Val::Px(5.)),
                ..Default::default()
            },
            background_color: ui::BackgroundColor(Color::linear_rgb(0.05, 0.05, 0.15)),
            border_color: ui::BorderColor(Color::linear_rgb(0.8, 0.6, 0.2)),
            visibility: Visibility::Hidden,
            focus_policy: ui::FocusPolicy::Block,
            ..Default::default()
        },
        ContainerNode,
        view::Owned,
        debug::Bundle::new("Infobox"),
    ));
}

/// Marker component for the container node for the info panel.
#[derive(Component)]
struct ContainerNode;

/// Marks that the entity is a `NodeBundle` for the specified viewable.
#[derive(Component)]
struct ViewableInfo(Entity);

/// Marks that the entity is the outermost `ViewableInfo` entity.
#[derive(Component)]
struct RootInfo;

/// Marker component for the label display node.
#[derive(Component)]
struct LabelDisplay;

fn update_hierarchy_system(
    mut commands: Commands,
    mut focus_change_events: EventReader<FocusChangeEvent>,
    focus: Res<Focus>,
    root_info_query: Query<Entity, With<RootInfo>>,
    container_query: Query<Entity, With<ContainerNode>>,
    viewable_children_query: Query<
        Option<&hierarchy::Children>,
        With<delegate::Marker<viewable::Sid>>,
    >,
) {
    // drain all events
    if focus_change_events.read().count() == 0 {
        return;
    }

    // recreate the hierarchy
    if let Ok(entity) = root_info_query.get_single() {
        commands.entity(entity).despawn_recursive();
    }

    let Some(focus) = focus.entity else {
        return; // no need to spawn anything
    };

    let root_info = spawn_hierarchy(
        &mut commands,
        container_query.single(),
        focus,
        &viewable_children_query,
        0,
    )
    .expect("0 < HIERARCHY_LAYERS && focus entity must exist");
    commands.entity(root_info).insert(RootInfo);
}

fn spawn_hierarchy(
    commands: &mut Commands,
    parent_entity: Entity,
    viewable_entity: Entity,
    viewable_children_query: &Query<
        Option<&hierarchy::Children>,
        With<delegate::Marker<viewable::Sid>>,
    >,
    depth: Depth,
) -> Option<Entity> {
    if depth >= HIERARCHY_LAYERS {
        return None;
    }

    let Ok(children_opt) = viewable_children_query.get(viewable_entity) else { return None };
    let children: Vec<Entity> = children_opt.into_iter().flatten().copied().collect();

    let new_entity = commands
        .spawn((
            ViewableInfo(viewable_entity),
            NodeBundle {
                style: Style {
                    flex_direction: ui::FlexDirection::Column,
                    margin: UiRect::ZERO.with_top(ui::Val::Px(3.)),
                    ..Default::default()
                },
                ..Default::default()
            },
            debug::Bundle::new("Infobox/Viewable"),
        ))
        .with_children(|b| {
            b.spawn((
                ViewableInfo(viewable_entity),
                LabelDisplay,
                TextBundle {
                    text: Text {
                        sections: vec![TextSection::new(
                            "",
                            TextStyle {
                                font_size: 20.0 - 2. * f32::from(depth),
                                ..Default::default()
                            },
                        )],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                debug::Bundle::new("Infobox/Viewable/Label"),
            ));
            metrics::spawn_ui(b, viewable_entity);
        })
        .id();
    commands.entity(parent_entity).add_child(new_entity);

    let children_container = commands
        .spawn((
            NodeBundle {
                style: Style {
                    margin: UiRect::ZERO.with_left(ui::Val::Px(5.)),
                    flex_direction: ui::FlexDirection::Column,
                    ..Default::default()
                },
                ..Default::default()
            },
            debug::Bundle::new("Infobox/Viewable/Children"),
        ))
        .id();
    commands.entity(new_entity).add_child(children_container);

    for child in children {
        spawn_hierarchy(commands, children_container, child, viewable_children_query, depth + 1);
    }

    Some(new_entity)
}

fn update_box_visibility_system(
    focus: Res<Focus>,
    mut container_query: Query<&mut Visibility, With<ContainerNode>>,
) {
    if let Ok(mut vis) = container_query.get_single_mut() {
        if focus.entity.is_some() {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn update_viewable_label_system(
    mut viewable_info_query: Query<(&ViewableInfo, &mut Text), With<LabelDisplay>>,
    object_query: Query<&Appearance, With<delegate::Marker<viewable::Sid>>>,
) {
    for (&ViewableInfo(viewable_entity), mut display) in &mut viewable_info_query {
        if let Ok(appearance) = object_query.get(viewable_entity) {
            let section = display.sections.get_mut(0).expect("set during init");
            section.value.clear();
            appearance.label.render(&mut section.value);
        } else {
            bevy::log::warn!(
                "missing appearance in viewable delegate {viewable_entity:?} referenced by \
                 viewable info"
            );
        }
    }
}

#[derive(Debug, Resource)]
pub struct Focus {
    pub entity:     Option<Entity>,
    pub focus_type: FocusType,
}

#[derive(Debug)]
pub enum FocusType {
    /// The current focused object, if any, was focused through hovering,
    /// and can be unfocused by moving the hover out.
    Hover,
    /// The current focused object was focused through explicit clicking,
    /// and must be unfocused by explicitly clicking outside.
    Locked,
}

pub(super) fn object_bundle() -> impl Bundle {
    (
        PickableBundle::default(),
        On::<Pointer<pick::Over>>::run(on_object_over),
        On::<Pointer<pick::Out>>::run(on_object_out),
    )
}

#[derive(Default, Event)]
struct FocusChangeEvent;

fn on_object_over(
    event: Listener<Pointer<pick::Over>>,
    mut focus: ResMut<Focus>,
    parent_query: Query<&hierarchy::Parent>,
    delegate_query: Query<(), With<delegate::Marker<viewable::Sid>>>,
    mut focus_change_writer: EventWriter<FocusChangeEvent>,
) {
    if let FocusType::Hover = focus.focus_type {
        let delegate = parent_query
            .iter_ancestors(event.target)
            .find(|&ancestor| delegate_query.get(ancestor).is_ok());
        if let Some(delegate) = delegate {
            focus.entity = Some(delegate);
        }
    }

    focus_change_writer.send_default();
}

fn on_object_out(
    event: Listener<Pointer<pick::Out>>,
    mut focus: ResMut<Focus>,
    parent_query: Query<&hierarchy::Parent>,
    delegate_query: Query<(), With<delegate::Marker<viewable::Sid>>>,
    mut focus_change_writer: EventWriter<FocusChangeEvent>,
) {
    if let FocusType::Hover = focus.focus_type {
        for ancestor in parent_query.iter_ancestors(event.target) {
            if delegate_query.get(ancestor).is_ok() && focus.entity == Some(ancestor) {
                focus.entity = None;
            }
        }
    }

    focus_change_writer.send_default();
}
