use core::fmt;
use std::marker::PhantomData;

use bevy::app::{self, App};
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::{Event, EventReader};
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Query, Res, ResMut, Resource};
use bevy::ecs::world::{Command, World};
use bevy::hierarchy::{BuildChildren, DespawnRecursiveExt};
use bevy::state::app::AppExtStates;
use bevy::state::state::{self, NextState};
use bevy::text::{JustifyText, Text, TextStyle};
use bevy::ui::node_bundles::{ButtonBundle, NodeBundle, TextBundle};
use bevy::ui::{self, BackgroundColor, Style, UiRect};
use traffloat_base::generic_state;
use typed_builder::TypedBuilder;

use crate::util::button;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Active {
    #[default]
    Inactive,
    Active,
}

generic_state!(pub ActiveState<But: Buttons>(Active));

/// Displays a modal dialog.
#[derive(TypedBuilder)]
pub struct DisplayCommand<But: Buttons> {
    #[builder(setter(into))]
    background_color: Color,
    #[builder(setter(into))]
    title:            String,
    #[builder(setter(into))]
    text:             String,
    #[builder(default, setter(skip))]
    _ph:              PhantomData<fn(But) -> But>,
}

impl<But: Buttons> Command for DisplayCommand<But> {
    fn apply(self, world: &mut World) {
        world.resource_mut::<NextState<ActiveState<But>>>().set(Active::Active.into());
        let mut param = world.resource_mut::<Param<But>>();
        param.background_color = self.background_color;
        param.title = self.title;
        param.text = self.text;
    }
}

#[derive(Resource)]
struct Param<But> {
    background_color: Color,
    title:            String,
    text:             String,
    _ph:              PhantomData<fn(But) -> But>,
}

impl<But> Default for Param<But> {
    fn default() -> Self {
        Self {
            background_color: <_>::default(),
            title:            <_>::default(),
            text:             <_>::default(),
            _ph:              PhantomData,
        }
    }
}

/// Buttons for a modal dialog.
pub trait Buttons: fmt::Debug + Copy + Send + Sync + Eq + 'static {
    /// Lists all buttons to display.
    fn iter() -> impl Iterator<Item = Self>;

    /// The name of the button.
    fn label(&self) -> String;

    /// Whether the button closes the modal.
    fn closing(&self) -> bool { true }
}

pub struct Plugin<But: Buttons>(PhantomData<fn(But) -> But>);

impl<But: Buttons> Default for Plugin<But> {
    fn default() -> Self { Self(PhantomData) }
}

impl<But: Buttons> app::Plugin for Plugin<But> {
    fn build(&self, app: &mut App) {
        app.init_state::<ActiveState<But>>();
        app.add_systems(state::OnEnter(ActiveState::<But>::from(Active::Active)), setup::<But>);
        app.add_systems(state::OnExit(ActiveState::<But>::from(Active::Active)), cleanup::<But>);
        app.add_plugins(button::Plugin::<ClickEvent<But>>::default());
        app.add_systems(
            app::Update,
            handle_button_click::<But>
                .in_set(button::HandleClickSystemSet::<ClickEvent<But>>::default()),
        );
        app.init_resource::<Param<But>>();
    }
}

#[derive(Component)]
struct RootNode<But>(PhantomData<But>);

fn setup<But: Buttons>(mut commands: Commands, param: Res<Param<But>>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: ui::Val::Percent(100.),
                    height: ui::Val::Percent(100.),
                    justify_content: ui::JustifyContent::Center,
                    justify_items: ui::JustifyItems::Center,
                    align_content: ui::AlignContent::Center,
                    align_items: ui::AlignItems::Center,
                    ..<_>::default()
                },
                ..<_>::default()
            },
            RootNode(PhantomData::<But>),
        ))
        .with_children(|builder| {
            builder
                .spawn(NodeBundle {
                    background_color: BackgroundColor(param.background_color),
                    style: Style {
                        justify_content: ui::JustifyContent::Center,
                        justify_items: ui::JustifyItems::Center,
                        align_content: ui::AlignContent::Center,
                        align_items: ui::AlignItems::Center,
                        flex_direction: ui::FlexDirection::Column,
                        width: ui::Val::Percent(50.),
                        height: ui::Val::Percent(30.),
                        padding: UiRect::axes(ui::Val::Px(30.), ui::Val::Px(0.)),
                        ..<_>::default()
                    },
                    ..<_>::default()
                })
                .with_children(|builder| {
                    builder.spawn(TextBundle {
                        text: Text::from_section(
                            &param.title,
                            TextStyle { font_size: 32., ..<_>::default() },
                        ),
                        style: Style {
                            margin: UiRect {
                                bottom: ui::Val::Px(50.),
                                ..UiRect::all(ui::Val::Px(0.))
                            },
                            ..<_>::default()
                        },
                        ..<_>::default()
                    });
                    builder.spawn(TextBundle {
                        text: Text::from_section(&param.text, TextStyle::default()),
                        style: Style { align_self: ui::AlignSelf::Start, ..<_>::default() },
                        ..<_>::default()
                    });

                    for button in But::iter() {
                        builder
                            .spawn(button::Bundle {
                                button: ButtonBundle {
                                    style: Style {
                                        padding: UiRect::all(ui::Val::Px(5.)),
                                        ..<_>::default()
                                    },
                                    ..<_>::default()
                                },
                                ..button::Bundle::new(ClickEvent(button))
                            })
                            .with_children(|builder| {
                                builder.spawn(TextBundle {
                                    text: Text::from_section(button.label(), TextStyle::default())
                                        .with_justify(JustifyText::Center),
                                    style: Style {
                                        width: ui::Val::Percent(100.),
                                        justify_content: ui::JustifyContent::Center,
                                        ..<_>::default()
                                    },
                                    ..<_>::default()
                                });
                            });
                    }
                });
        });
}

fn cleanup<But: Buttons>(mut commands: Commands, query: Query<Entity, With<RootNode<But>>>) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_button_click<But: Buttons>(
    mut events: EventReader<ClickEvent<But>>,
    mut next_state: ResMut<NextState<ActiveState<But>>>,
) {
    for ev in events.read() {
        if ev.0.closing() {
            next_state.set(ActiveState::from(Active::Inactive));
        }
    }
}

/// Fired when a button is clicked.
#[derive(Debug, Clone, Event)]
pub struct ClickEvent<But: Buttons>(pub But);
