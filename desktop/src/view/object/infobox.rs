use bevy::app::{self, App};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::{Event, EventWriter};
use bevy::ecs::query::With;
use bevy::ecs::schedule::{IntoSystemConfigs, SystemSet};
use bevy::ecs::system::{Query, Res, ResMut, Resource};
use bevy::hierarchy::{self, HierarchyQueryExt};
use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::state::condition::in_state;
use bevy_egui::{egui, EguiContexts};
use bevy_eventlistener::callbacks::Listener;
use bevy_eventlistener::event_listener::On;
use bevy_mod_picking::prelude::{self as pick, Pointer};
use bevy_mod_picking::PickableBundle;
use traffloat_base::partition::AppExt;
use traffloat_base::{ClientSideSystemSet, UiMutatorSystemSet};
use traffloat_view::{viewable, Appearance};

use super::metrics;
use crate::util::glossary;
use crate::view::delegate;
use crate::AppState;

type Depth = u16;

const HIERARCHY_LAYERS: Depth = 3;

pub(super) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Focus>();
        app.add_partitioned_event::<FocusChangeEvent>();
        app.add_systems(
            app::Update,
            update_hierarchy_system
                .in_set(ClientSideSystemSet)
                .in_set(UiMutatorSystemSet)
                .run_if(in_state(AppState::GameView))
                .in_set(RenderSystemSet)
                .after(
                    delegate::SidIndexMaintainerSystemSet::<traffloat_view::metrics::Sid>::default(
                    ),
                ),
        );
        app.add_systems(app::First, handle_mouse_event_system);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub(super) struct RenderSystemSet;

fn update_hierarchy_system(
    focus: Res<Focus>,
    mut egui_ctxs: EguiContexts,
    viewable_query: Query<
        (&Appearance, Option<&hierarchy::Children>),
        With<delegate::Marker<viewable::Sid>>,
    >,
    glossary_provider: glossary::Provider,
    metrics_params: metrics::RenderUiParams,
) {
    let Some(ctx) = egui_ctxs.try_ctx_mut() else { return };

    let Some(focus) = focus.current_focus() else {
        return; // no need to spawn anything
    };

    egui::SidePanel::right("infobox").resizable(true).show(ctx, |ui| {
        render_hierarchy(ui, focus, &viewable_query, &glossary_provider, &metrics_params, 0);
    });
}

fn render_hierarchy(
    ui: &mut egui::Ui,
    viewable_entity: Entity,
    viewable_query: &Query<
        (&Appearance, Option<&hierarchy::Children>),
        With<delegate::Marker<viewable::Sid>>,
    >,
    glossary_provider: &glossary::Provider,
    metrics_params: &metrics::RenderUiParams,
    depth: Depth,
) {
    if depth >= HIERARCHY_LAYERS {
        return;
    }

    let Ok((appearance, children_opt)) = viewable_query.get(viewable_entity) else { return };
    let children: Vec<Entity> = children_opt.into_iter().flatten().copied().collect();

    ui.label(appearance.label.render_to_string(glossary_provider, &[]));

    metrics::render_ui(ui, viewable_entity, metrics_params);

    for child in children {
        render_hierarchy(ui, child, viewable_query, glossary_provider, metrics_params, depth + 1);
    }
}

#[derive(Debug, Default, Resource)]
pub struct Focus {
    locked_target:  Option<Entity>,
    locking_target: Option<Entity>,
    current_hover:  Option<Entity>,
}

impl Focus {
    pub fn current_focus(&self) -> Option<Entity> {
        self.locked_target.or(self.locking_target).or(self.current_hover)
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HasChanged {
    NewValue,
    SameValue,
}

fn compare_and_assign<T: PartialEq>(target: &mut T, value: T) -> HasChanged {
    if *target == value {
        HasChanged::SameValue
    } else {
        *target = value;
        HasChanged::NewValue
    }
}

fn on_object_over(
    event: Listener<Pointer<pick::Over>>,
    mut focus: ResMut<Focus>,
    parent_query: Query<&hierarchy::Parent>,
    delegate_query: Query<(), With<delegate::Marker<viewable::Sid>>>,
    mut focus_change_writer: EventWriter<FocusChangeEvent>,
) {
    let delegate = parent_query
        .iter_ancestors(event.target)
        .find(|&ancestor| delegate_query.get(ancestor).is_ok());

    let changed = compare_and_assign(&mut focus.current_hover, delegate);
    if changed == HasChanged::NewValue {
        focus_change_writer.send_default();
    }
}

fn on_object_out(
    event: Listener<Pointer<pick::Out>>,
    mut focus: ResMut<Focus>,
    parent_query: Query<&hierarchy::Parent>,
    delegate_query: Query<(), With<delegate::Marker<viewable::Sid>>>,
    mut focus_change_writer: EventWriter<FocusChangeEvent>,
) {
    let target = parent_query
        .iter_ancestors(event.target)
        .find(|&ancestor| delegate_query.get(ancestor).is_ok());

    if target.is_some() && focus.current_hover == target {
        focus.current_hover = None;

        focus_change_writer.send_default();
    }
}

fn handle_mouse_event_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut focus: ResMut<Focus>,
    mut focus_change_writer: EventWriter<FocusChangeEvent>,
) {
    // Call ResMut::deref_mut exactly once
    // to allow borrow checker to destructure fields as separate mutable references.
    let focus = &mut *focus;

    if mouse_buttons.just_pressed(MouseButton::Left) {
        focus.locking_target = focus.current_hover;
    }

    if mouse_buttons.just_released(MouseButton::Left) && focus.locking_target == focus.current_hover
    {
        let changed = compare_and_assign(&mut focus.locked_target, focus.current_hover);
        if changed == HasChanged::NewValue {
            focus_change_writer.send_default();
        }
    }
}
